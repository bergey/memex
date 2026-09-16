use memex_shared as memex;
use phish::*;
use automerge::sync::SyncDoc;

use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{error, warn};
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

async fn act(ws_stream: WSS, mut timer: Timer) {
    let (mut write, mut read) = ws_stream.split();

    // consume & discard all incoming messages
    spawn(async move {
        loop {
            let _ = read.next().await;
        }
    });

    timer.next().unwrap().await;
    let library_id = memex::LibraryId::random();
    send_message(&mut write, memex::Message::LibraryId(library_id)).await;
    let mut library = memex::Library::new(automerge::ActorId::random());
    library.id = library_id;
    let mut sync_state = automerge::sync::State::new();

    while let Some(future) = timer.next() {
        future.await;
        // TODO more interesting messages
        let _r_id = library.add_record();
        if let Some(am_msg) = library.replicated.sync().generate_sync_message(&mut sync_state) {
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
