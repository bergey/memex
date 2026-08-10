mod observability;
mod prelude;

use prelude::*;

use automerge::AutoCommit;
use automerge::sync::SyncDoc;
use axum::{
    Router,
    extract::{
        State,
        ws::{self, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::get,
};
use sqlx::postgres::PgPoolOptions;
// use tokio::try_join;

fn ws_upgrade(pools: Pools, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |ws| {
        let pools = pools.clone();
        let sync_state = automerge::sync::State::new();
        ws_loop(ws, move |msg| save_message(pools, sync_state, msg))
    })
}

// TODO cache in memory, don't read on every edit
async fn save_message(
    State(_database): Pools,
    mut sync_state: automerge::sync::State,
    ws_msg: ws::Message,
) -> anyhow::Result<Option<ws::Message>> {
    let message = decode_message(ws_msg)?;
    // let document = read_library(message.id)
    //     .await?
    //     .unwrap_or_else(|| AutoCommit::new());
    let mut document = AutoCommit::new(); // TODO remove
    document
        .sync()
        .receive_sync_message(&mut sync_state, message)?;
    Ok(None) // TODO reply?
}

async fn ws_loop<F, Fut>(mut socket: WebSocket, mut handle: F)
where
    F: FnMut(ws::Message) -> Fut,
    Fut: Future<Output = anyhow::Result<Option<ws::Message>>>,
{
    loop {
        let o_msg = socket.recv().await;
        match o_msg {
            None => break, // client closed TODO metrics
            Some(Err(e)) => {
                error!("in recv: {e}");
                break;
            }
            Some(Ok(ws_msg)) => match handle(ws_msg).await {
                Err(e) => {
                    error!("{e}");
                    break;
                }
                Ok(None) => {}
                Ok(Some(reply)) => {
                    if let Err(e) = socket.send(reply).await {
                        error!("in send: {e}");
                        break;
                    }
                }
            },
        }
    }
}

fn decode_message(ws_msg: ws::Message) -> anyhow::Result<automerge::sync::Message> {
    match ws_msg {
        ws::Message::Binary(bytes) => Ok(automerge::sync::Message::decode(bytes.as_ref())?),
        _ => Err(anyhow::anyhow!("expected binary WS message"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    observability::init()?;

    let connection_pools = ConnectionPools {
        postgres: PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://memex:memex@localhost:5432/memex")
            .await?,
    };

    let app = Router::new()
        .route("/ws", get(ws_upgrade))
        .with_state(connection_pools);
    // TODO serve client code

    let addr = "0.0.0.0:3036"; // TODO env var
    info!("memex listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
