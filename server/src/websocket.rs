use crate::database;
use crate::prelude::*;

use automerge::AutoCommit;
use automerge::sync::SyncDoc;
use axum::{
    extract::{
        State,
        ws::{self, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;

lazy_static! {
    static ref BROADCAST: Sender<AutoCommit> = broadcast::channel(1024).0;
}

pub async fn ws_upgrade(pools: Pools, ws: WebSocketUpgrade) -> Response {
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
                    Some(Ok(ws_msg)) => match apply_message_reply(&pools, &mut sync_state, ws_msg).await {
                        Err(e) => {
                            error!("{e}");
                            break;
                        }
                        Ok(None) => {}
                        Ok(Some(reply)) => {
                            info!("sending reply");
                            if let Err(e) = socket.send(reply).await {
                                error!("in send: {e}");
                                break;
                            }
                        }
                    },
                }
            },
            Ok(doc) = other_clients.recv() => {
                let mut doc = doc.clone();
                let heads = doc.get_heads();
                match doc.sync().generate_sync_message(&mut sync_state) {
                    None => debug!( sync_state = ?sync_state, our_heads = ?heads, "not forwarding"),
                    Some(reply) =>
                    {
                        info!("forwarding remote edit");
                        if let Err(e) = socket.send(ws::Message::Binary(reply.encode().into())).await {
                            error!("in broadcast send: {e}");
                            break;
                        }
                    }
                }
            },
            else => break
        }
    }
}

async fn apply_message_reply(
    State(database): &Pools,
    sync_state: &mut automerge::sync::State,
    ws_msg: ws::Message,
) -> anyhow::Result<Option<ws::Message>> {
    info!("received WS message");
    let message = decode_message(ws_msg)?;
    let id = Uuid::nil(); // TODO ID from message / history
    let mut document = database::apply_message(&database.postgres, id, sync_state, message).await?;
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
