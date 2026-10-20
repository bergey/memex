mod connect;
mod server;

use crate::prelude::*;
use memex_shared::{
  library::{Library, action::Action},
  message::Message,
};

use automerge::sync::SyncDoc;
use futures::channel::mpsc::Receiver;
use futures::select;
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::WebSocket;

#[derive(Debug, Clone, Copy)]
pub enum ConnStatus {
  Connected,
  Disconnected,
}

#[derive(Debug)]
pub struct ServerSync {
  url: String,
  ws: WebsocketBackoff,
  tx: Sender<Message>,
  tx_c: Sender<ConnStatus>,
}

#[derive(Debug, Clone)]
pub enum WebsocketBackoff {
  Connected(WebSocket),
  Backoff(u32),
}

static ACTION_Q: LazyLock<Mutex<Option<Sender<Action>>>> = LazyLock::new(|| Mutex::new(None));

#[wasm_bindgen]
pub async fn setup_shared(server_ws_url: Option<String>) {
  setup_observability();
  // Network
  let (tx_up, rx_down, rx_c) = if let Some(ws_url) = server_ws_url {
    ServerSync::start(ws_url)
  } else {
    let (tx_up, _) = Sender::new(0);
    let (tx_down, rx_down) = Sender::new(0);
    let (tx_c, rx_c) = Sender::new(0);
    std::mem::forget(tx_down);
    std::mem::forget(tx_c);
    (tx_up, rx_down, rx_c)
  };

  // Library
  // TODO send library to main thread?
  let library = crate::disk::load_some_library().await;
  let mut lock = ACTION_Q.lock().unwrap();
  if lock.is_none() {
    let (tx_a, rx_a) = Sender::new(3);
    *lock = Some(tx_a);
    spawn_local(shared_worker_thread(library, rx_a, rx_down, rx_c, tx_up));
  }
}

#[wasm_bindgen]
pub fn connect_tab(port: web_sys::MessagePort) {
  let cb = action_callback();
  port.set_onmessage(Some(cb.as_ref().unchecked_ref()));
  cb.forget(); // TODO avoid leaking?
}

fn action_callback() -> Closure<dyn FnMut(web_sys::MessageEvent)> {
  let lock = ACTION_Q.lock().unwrap();
  let mut tx = match &*lock {
    Some(tx) => {
      tx.clone()
    }
    None => {
      error!("action_callback before setup_shared");
      Sender::new(0).0
    }
  };
  Closure::new(move |ev: web_sys::MessageEvent| {
    if let Some(action) = serde_wasm_bindgen::from_value(ev.data()).log_error() {
      tx.send(action);
    }
  })
}

async fn shared_worker_thread(
  mut library: Library,
  mut rx_a: Receiver<Action>,
  mut rx_down: Receiver<Message>,
  mut rx_c: Receiver<ConnStatus>,
  mut tx_up: Sender<Message>,
) {
  let mut sync_state = automerge::sync::State::new();
  let mut connected = false;
  let mut broadcaster = ToWorker::broadcast("shared_worker");
  loop {
    select! {
       r_action = rx_a.recv() => {
            let action = r_action.context("rx_action").unwrap();
           let patches = library.apply(&action);
           crate::disk::save_library(&mut library);
           for p in patches {
               broadcaster.send(&p);
           }
           if connected {
               if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                   tx_up.send(Message::Library(message))
               }
           }
       },

        r_down = rx_down.recv() => {
            use Message::*;
            match r_down.context("rx_down").unwrap() {
                Library(message) => {
                    library.replicated.sync().receive_sync_message(&mut sync_state, message).unwrap();
                    for p in library.get_patches(){
                        broadcaster.send(&p);
                    }
                    let our_heads = library.replicated.get_heads();
                    if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                        tx_up.send(Message::Library(message));
                    } else {
                        debug!( ?sync_state, ?our_heads, "no reply");
                    }
                }

                LibraryId(_id) => {// TODO
                    warn!("for now we only support one Library per user");
                }

                Authorize(_) => {
                    warn!("should only be sent from client to server")
                }
            }
        },

        r_c = rx_c.recv() => {
            use ConnStatus::*;
            match r_c.context("r_c").unwrap() {
                Connected => {
                    connected = true;
                    tx_up.send(Message::LibraryId(library.id));
                },
                Disconnected => connected = false,
            }
        }

    }
  }
}
