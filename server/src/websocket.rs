use crate::database;
use crate::metrics;
use crate::observability::hist_time_since;
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
use std::time::Instant;
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;

lazy_static! {
    static ref BROADCAST: Sender<AutoCommit> = broadcast::channel(1024).0;
}

pub async fn ws_upgrade(pools: Pools, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| sync_crdt_ws(pools, ws))
}

async fn sync_crdt_ws(pools: Pools, mut socket: WebSocket) {
    metrics::WS_CONNECT.inc();
    let mut sync_state = automerge::sync::State::new();
    let mut other_clients = BROADCAST.subscribe();

    loop {
        tokio::select! {
            o_msg = socket.recv() => {
                match o_msg {
                    None => break, // client closed TODO metrics
                    Some(Err(e)) => {
                        warn!("in recv: {e}");
                        break;
                    }
                    Some(Ok(ws_msg)) => {
                        let start = Instant::now();
                        match apply_message_reply(&pools, &mut sync_state, ws_msg).await {
                            Err(e) => {
                                warn!("{e}");
                                break;
                            }
                            Ok(None) => { info!("received WS message, no reply") }
                            Ok(Some(reply)) => {
                                info!("sending WS reply");
                                if let Err(e) = socket.send(reply).await {
                                    metrics::WS_SEND_ERROR.inc();
                                    warn!("in send: {e}");
                                    break;
                                }
                            }
                        }
                        hist_time_since(&*metrics::WS_MESSAGE_LATENCY, start);
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
    metrics::WS_DISCONNECT.inc();
}

async fn apply_message_reply(
    State(database): &Pools,
    sync_state: &mut automerge::sync::State,
    ws_msg: ws::Message,
) -> anyhow::Result<Option<ws::Message>> {
    metrics::AUTOMERGE_MESSAGE_RECEIVED.inc();
    let message =
        decode_message(ws_msg).inspect_err(|_| metrics::GARBAGE_MESSAGE_RECEIVED.inc())?;
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
