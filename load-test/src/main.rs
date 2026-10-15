use automerge::sync::SyncDoc;
use memex_shared as memex;
use phish::*;

#[macro_use]
extern crate lazy_static;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{error, info, warn};
use prometheus::{self, Histogram};
use rand::distr::SampleString;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::time::Instant;
use tokio::{spawn, sync::mpsc};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

// TODO how to combine args from phish, application-specific

type WSS = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct User {
  token: memex::user::AuthToken,
  ws: WSS,
}

#[tokio::main(worker_threads = 16)]
async fn main() -> anyhow::Result<()> {
  phish_main(connect, act).await?;
  report_metrics_and_exit(0);
  Ok(())
}

lazy_static! {
  static ref ROUND_TRIP_TIME: Histogram = prometheus::register_histogram!(
    "round_trip_time",
    "client-side time to persist one new line to server"
  )
  .unwrap();
}

async fn connect(user_id: usize) -> User {
  loop {
    // TODO WS URL from args
    match connect_async("ws://prandtl:8810/ws").await {
      Ok((ws, _)) => {
        break User {
          ws,
          token: memex::user::AuthToken(user_id as u128),
        };
      }
      Err(err) => {
        // no backoff, trust whoever runs the load test to adjust the ramp if necessary
        warn!("user {user_id}: failed to connect, will retry: {err}");
      }
    }
  }
}

struct Shared {
  library: memex::Library,
  sync_state: automerge::sync::State,
}

impl Shared {
  // these are functions so a single lifetime is inferred for both fields
  fn generate_sync_message(&mut self) -> Option<automerge::sync::Message> {
    self
      .library
      .replicated
      .sync()
      .generate_sync_message(&mut self.sync_state)
  }

  fn receive_sync_message(&mut self, am_msg: automerge::sync::Message) {
    let _ = self
      .library
      .replicated
      .sync()
      .receive_sync_message(&mut self.sync_state, am_msg);
  }
}

async fn act(user: User, mut timer: Timer) {
  let (mut write, mut read) = user.ws.split();
  let (tx, mut rx) = mpsc::channel(30);

  // TODO consider moving this into connect / User
  let (library_id, shared) = {
    let mut library = memex::Library::new(automerge::ActorId::random());
    let library_id = memex::LibraryId::random();
    library.id = library_id;
    let sync_state = automerge::sync::State::new();
    let shared = Arc::new(Mutex::new(Shared {
      library,
      sync_state,
    }));
    (library_id, shared)
  };

  // consume & apply all incoming messages
  let shared_for_read = shared.clone();
  spawn(async move {
    loop {
      if let Some(Ok(tungstenite::Message::Binary(ws_bytes))) = read.next().await {
        if let Ok(memex::Message::Library(am_msg)) = memex::Message::decode(&ws_bytes) {
          if let Some(start_time) = rx.recv().await {
            let elapsed = Instant::now().duration_since(start_time);
            ROUND_TRIP_TIME.observe(elapsed.as_secs_f64());
          }
          let mut shared = shared_for_read.lock().unwrap();
          shared.receive_sync_message(am_msg);
        }
      }
    }
  });

  match timer.next() {
    Some(sleep) => sleep.await,
    None => return, // other users satisfied all requests
  }
  send_message(&mut write, memex::Message::Authorize(user.token), &tx).await;
  send_message(&mut write, memex::Message::LibraryId(library_id), &tx).await;
  let mut i = 1;
  let mut r_id = {
    let mut lock = shared.lock().unwrap();
    lock.library.add_record()
  };
  let mut s = String::new();

  while let Some(future) = timer.next() {
    future.await;
    let o_msg = {
      let mut shared = shared.lock().unwrap();
      randomize_string(&mut s);
      i = (i + 1) % 4;
      match i {
        0 => r_id = shared.library.add_record(),
        1 => shared.library.set_title(&r_id, &s).unwrap(),
        2 => shared.library.set_author(&r_id, &s).unwrap(),
        3 => shared.library.set_url(&r_id, &s).unwrap(),
        _ => error!("i out of bounds: {i}"),
      }
      shared.generate_sync_message()
    };
    if let Some(am_msg) = o_msg {
      send_message(&mut write, memex::Message::Library(am_msg), &tx).await;
    }
  }
}

async fn send_message(
  write: &mut SplitSink<WSS, tungstenite::Message>,
  msg: memex::Message,
  tx: &mpsc::Sender<Instant>,
) {
  let r = write
    .send(tungstenite::Message::Binary(
      tungstenite::Bytes::from_owner(msg.encode()),
    ))
    .await;
  if let Err(e) = r {
    error!("{:?}", e);
  }
  let _ = tx.send(Instant::now()).await;
}

fn randomize_string(s: &mut String) {
  rand::distr::Alphabetic.append_string(&mut rand::rng(), s, 8);
}

fn report_metrics_and_exit(exit_code: i32) {
  let metrics = prometheus::gather();
  let report = prometheus::TextEncoder::new()
    .encode_to_string(&metrics)
    .expect("failed to encode metrics");
  info!("{report}");
  // TODO graceful WS close
  std::process::exit(exit_code);
}
