mod database;
mod observability;
mod prelude;

use database::*;
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
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref BROADCAST: Sender<AutoCommit> =
        broadcast::channel(1024).0;
}

async fn ws_upgrade(pools: Pools, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| sync_crdt_ws(pools, ws))
}

async fn sync_crdt_ws(pools: Pools, mut socket: WebSocket) {
    let mut sync_state = automerge::sync::State::new();
    let mut other_clients = BROADCAST.subscribe();

    loop {
        tokio::select! {
            o_msg = socket.recv() => {
                match o_msg {
                    None => break, // client closed TODO metrics
                    Some(Err(e)) => {
                        error!("in recv: {e}");
                        break;
                    }
                    Some(Ok(ws_msg)) => match save_message(&pools, &mut sync_state, ws_msg).await {
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
            },
            Ok(mut doc) = other_clients.recv() => {
                if let Some(reply) = doc.sync().generate_sync_message(&mut sync_state) {
                    if let Err(e) = socket.send(ws::Message::Binary(reply.encode().into())).await {
                        error!("in broadcast send: {e}");
                        break;
                    }
                }
            },
            else => break
        }
    }
}

async fn save_message(
    State(database): &Pools,
    sync_state: &mut automerge::sync::State,
    ws_msg: ws::Message,
) -> anyhow::Result<Option<ws::Message>> {
    let message = decode_message(ws_msg)?;
    let id = Uuid::nil(); // TODO ID from message / history
    let mut transaction = database.postgres.begin().await?;
    let mut document = read_library(&mut transaction, id)
        .await?
        .unwrap_or_else(|| AutoCommit::new());
    document.sync().receive_sync_message(sync_state, message)?;
    save_library(&mut transaction, id, &mut document).await?;
    transaction.commit().await?;
    let _ = BROADCAST.send(document.clone());
    let o_reply = document.sync().generate_sync_message(sync_state);
    Ok(o_reply.map(|reply| ws::Message::Binary(reply.encode().into())))
}

fn decode_message(ws_msg: ws::Message) -> anyhow::Result<automerge::sync::Message> {
    match ws_msg {
        ws::Message::Binary(bytes) => Ok(automerge::sync::Message::decode(bytes.as_ref())?),
        _ => Err(anyhow::anyhow!("expected binary WS message")),
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
