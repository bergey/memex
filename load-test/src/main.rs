// use memex_shared as memex;
use phish::*;

use log::warn;
use tokio::net::TcpStream;
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

async fn act(_ws_stream: WSS, mut timer: Timer) {
    while let Some(future) = timer.next() {
        future.await;
    }
}
