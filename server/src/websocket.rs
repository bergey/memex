use crate::database;
use crate::metrics;
use crate::observability::hist_time_since;
use crate::prelude::*;
use memex_shared::{Library, LibraryId, Message};

use automerge::sync::SyncDoc;
use axum::{
    extract::{
        State,
        ws::{self, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::broadcast::{self, Sender};

lazy_static! {
    static ref BROADCAST: Sender<Library> = broadcast::channel(1024).0;
}

#[derive(Clone, Debug)]
struct Client {
    am: HashMap<LibraryId, automerge::sync::State>,
    library_out: Option<LibraryId>,
    library_in: Option<LibraryId>,
}

impl Client {
    fn new() -> Self {
        Client {
            am: HashMap::new(),
            library_out: None,
            library_in: None,
        }
    }

    fn get_am<'a>(&'a mut self, id: LibraryId) -> &'a mut automerge::sync::State {
        if !self.am.contains_key(&id) {
            self.am.insert(id, automerge::sync::State::new());
        }
        self.am.get_mut(&id).unwrap()
    }
}

pub async fn ws_upgrade(pools: Pools, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| sync_crdt_ws(pools, ws))
}

async fn sync_crdt_ws(pools: Pools, mut socket: WebSocket) {
    metrics::WS_CONNECT.inc();
    let mut other_clients = BROADCAST.subscribe();
    let mut client_state = Client::new();

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
                        match apply_message_reply(&pools, &mut client_state, ws_msg).await {
                            Err(e) => {
                                warn!("{e}");
                                break;
                            }
                            Ok(None) => { info!("received WS message, no reply") }
                            Ok(Some((id, reply))) => {
                                if try_send_library(&mut socket, reply, id, &mut client_state, "reply").await.is_err() {
                                    break;
                                }
                                info!("sending WS reply");
                            }
                        }
                        hist_time_since(&*metrics::WS_MESSAGE_LATENCY, start);
                    },
                }
            },

            Ok(doc) = other_clients.recv() => {
                let mut doc = doc.clone();
                let heads = doc.replicated.get_heads();
                match doc.replicated.sync().generate_sync_message(&mut client_state.get_am(doc.id)) {
                    None => debug!( sync_state = ?client_state, our_heads = ?heads, "not forwarding"),
                    Some(notice) =>
                    {
                        if try_send_library(&mut socket, Message::Library(notice), doc.id, &mut client_state, "broadcast").await.is_err() {
                            break;
                        }
                        info!("forwarding remote edit");
                    }
                }
            },
            else => break
        }
    }
    metrics::WS_DISCONNECT.inc();
}

// send a WS message about id.  Send a LibraryId message first if necessary
// Log errors, return Err so the outer loop can break
async fn try_send_library(
    socket: &mut WebSocket,
    message: Message,
    id: LibraryId,
    client_state: &mut Client,
    context: &str, // TODO some general tracing / event machinery instead of this argument
) -> anyhow::Result<()> {
    if Some(id) != client_state.library_out {
        let id_msg = Message::LibraryId(id);
        try_send(socket, id_msg, context).await?;
        client_state.library_out = Some(id);
    }
    try_send(socket, message, context).await?;
    Ok(())
}

// send one WS message.  Log errors, return Err so the outer loop can break
async fn try_send(socket: &mut WebSocket, message: Message, context: &str) -> anyhow::Result<()> {
    let ret = socket
        .send(ws::Message::Binary(message.encode().into()))
        .await;
    if let Err(e) = &ret {
        metrics::WS_SEND_ERROR.inc();
        warn!("{context}: {e}");
    }
    ret?;
    Ok(())
}

async fn apply_message_reply(
    State(database): &Pools,
    client_state: &mut Client,
    ws_msg: ws::Message,
) -> anyhow::Result<Option<(LibraryId, Message)>> {
    metrics::AUTOMERGE_MESSAGE_RECEIVED.inc();
    let message =
        decode_message(ws_msg).inspect_err(|_| metrics::GARBAGE_MESSAGE_RECEIVED.inc())?;
    match message {
        Message::Library(am_msg) => {
            let id = client_state
                .library_in
                .ok_or_else(|| anyhow::anyhow!("sync message before Library ID set"))?;
            let am_state = client_state.get_am(id);
            let mut document =
                database::apply_message(&database.postgres, id, am_state, am_msg)
                    .await?;
            let _ = BROADCAST.send(Library {
                id,
                replicated: document.clone(),
            });
            let o_reply = document.sync().generate_sync_message(am_state);
            Ok(o_reply.map(|msg| (id, Message::Library(msg))))
        }
        Message::LibraryId(id) => {
            client_state.library_in = Some(id);
            Ok(None)
        }
    }
}

fn decode_message(ws_msg: ws::Message) -> anyhow::Result<Message> {
    match ws_msg {
        ws::Message::Binary(bytes) => Ok(Message::decode(bytes.as_ref())?),
        _ => Err(anyhow::anyhow!("expected binary WS message")),
    }
}
