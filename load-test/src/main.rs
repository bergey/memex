use automerge::sync::SyncDoc;
use memex_shared as memex;
use phish::*;

use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{error, warn};
use rand::distr::SampleString;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

// TODO how to combine args from phish, application-specific

type WSS = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    phish_main(connect, act).await
}

async fn connect(user_id: usize) -> WSS {
    loop {
        match connect_async("http://localhost:3036").await {
            Ok((ws, _)) => break ws,
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

async fn act(ws_stream: WSS, mut timer: Timer) {
    let (mut write, mut read) = ws_stream.split();

    let (library_id, shared) = {
        let library_id = memex::LibraryId::random();
        let mut library = memex::Library::new(automerge::ActorId::random());
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
                    let mut shared = shared_for_read.lock().unwrap();
                    receive_sync_message(&mut shared, am_msg);
                }
            }
        }
    });

    timer.next().unwrap().await;
    send_message(&mut write, memex::Message::LibraryId(library_id)).await;
    let mut i = 0;
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
            generate_sync_message(&mut shared)
        };
        if let Some(am_msg) = o_msg
        {
            send_message(&mut write, memex::Message::Library(am_msg)).await;
        }
    }
}

async fn send_message(write: &mut SplitSink<WSS, tungstenite::Message>, msg: memex::Message) {
    let r = write
        .send(tungstenite::Message::Binary(
            tungstenite::Bytes::from_owner(msg.encode()),
        ))
        .await;
    if let Err(e) = r {
        error!("{:?}", e);
    }
}

fn randomize_string(s: &mut String) {
    rand::distr::Alphabetic.append_string(&mut rand::rng(), s, 8);
}

// these are functions so a single lifetime is inferred for both fields
fn generate_sync_message(shared: &mut Shared) -> Option<automerge::sync::Message> {
    shared.library
        .replicated
        .sync()
        .generate_sync_message(&mut shared.sync_state)
}

fn receive_sync_message(shared: &mut Shared, am_msg: automerge::sync::Message) {
    let _ = shared.library
        .replicated
        .sync()
        .receive_sync_message(&mut shared.sync_state, am_msg);
}
