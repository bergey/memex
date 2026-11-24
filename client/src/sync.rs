mod connect;
mod server;

use crate::prelude::*;
use memex_shared::AuthToken;
use memex_shared::{
  ToClient, ToServer,
  library::{self, Library},
};

use automerge::sync::SyncDoc;
use futures::channel::mpsc::Receiver;
use futures::select;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::WebSocket;

/// From browser tab to shared Worker
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ToWorker {
  Library(library::Action),
  Authenticate(AuthToken),
}

/// from shared worker to browser Tabs (broadcast)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ToTabs {
  Library(library::Event),
  Server(ConnStatus),
  NotAuthorized,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ConnStatus {
  Connected,
  Disconnected,
}

#[derive(Debug)]
pub struct ServerSync {
  url: String,
  ws: WebsocketBackoff,
  tx: Sender<ToClient>,
  tx_c: Sender<ConnStatus>,
}

#[derive(Debug, Clone)]
pub enum WebsocketBackoff {
  Connected(WebSocket),
  Backoff(u32),
}

static ACTION_Q: LazyLock<Mutex<Option<Sender<ToWorker>>>> = LazyLock::new(|| Mutex::new(None));

impl ServerSync {
  fn disconnected(&mut self) {
    self.ws = WebsocketBackoff::Backoff(0);
    self.tx_c.send(ConnStatus::Disconnected);
  }
}

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
    Some(tx) => tx.clone(),
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
  mut rx_t: Receiver<ToWorker>,
  mut rx_down: Receiver<ToClient>,
  mut rx_c: Receiver<ConnStatus>,
  mut tx_up: Sender<ToServer>,
) {
  let mut sync_state = automerge::sync::State::new();
  let mut connected = false;
  let mut broadcaster = BrowserPort::<ToTabs>::broadcast("shared_worker");
  loop {
    select! {
       r_from_tab = rx_t.recv() => {
         match  r_from_tab.context("rx_to_worker").unwrap() {
           ToWorker::Library(action) => {
           let patches = library.apply(&action);
           crate::disk::save_library(&mut library);
           for p in patches {
             broadcaster.send(&ToTabs::Library(p));
           }
           if connected {
               if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                   tx_up.send(ToServer::Library(message))
               }
           }
           },
           ToWorker::Authenticate(token) => {
             let outcome = crate::auth::save_auth_token(token).await;
             debug!(?outcome, "saved auth token");
             tx_up.send(ToServer::Authorize(token));
           }
         }
       },

        r_down = rx_down.recv() => {
            use ToClient::*;
            match r_down.context("rx_down").unwrap() {
                Library(message) => {
                    library.replicated.sync().receive_sync_message(&mut sync_state, message).unwrap();
                    for p in library.get_patches(){
                      broadcaster.send(&ToTabs::Library(p));
                    }
                    let our_heads = library.replicated.get_heads();
                    if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                        tx_up.send(ToServer::Library(message));
                    } else {
                        debug!( ?sync_state, ?our_heads, "no reply");
                    }
                }

                LibraryId(_id) => {// TODO
                    warn!("for now we only support one Library per user");
                }
            }
        },

        r_c = rx_c.recv() => {
            use ConnStatus::*;
            match r_c.context("r_c").unwrap() {
                Connected => {
                    connected = true;
                    tx_up.send(ToServer::LibraryId(library.id));
                },
                Disconnected => connected = false,
            }
        }

    }
  }
}
