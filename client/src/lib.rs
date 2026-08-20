mod actor;
mod disk;
mod prelude;
mod render;
mod sync;

use memex_shared::{
    library::{
        Library,
        action::{Action, Event},
    },
    message::Message,
};
use prelude::*;

use automerge::sync::SyncDoc;
use futures::channel::mpsc::Receiver;
use futures::select;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn start(server_ws_url: Option<String>) -> Result<()> {
    console_error_panic_hook::set_once();
    wasm_tracing::set_as_global_default();

    // from DOM to Library
    let (tx_a, rx_a) = Sender::new(3);
    // from Library to DOM
    let (tx_e, mut rx_e) = Sender::new(3);

    // Network
    let (tx_up, rx_down, rx_c) = if let Some(ws_url) = server_ws_url {
        sync::ServerSync::start(ws_url)
    } else {
        let (tx_up, _) = Sender::new(0);
        let (tx_down, rx_down) = Sender::new(0);
        let (tx_c, rx_c) = Sender::new(0);
        std::mem::forget(tx_down);
        std::mem::forget(tx_c);
        (tx_up, rx_down, rx_c)
    };

    spawn_local((async move || {
        let library = disk::load_some_library().await;
        let reactive = render::ReactiveLibrary::from_replicated(&library);
        let mut reactive_for_updates = reactive.clone();

        // DOM updates
        spawn_local(async move {
            loop {
                match rx_e.recv().await {
                    Ok(action) => reactive_for_updates.apply(action),
                    Err(_) => break, // TODO should raise a JS exception, reload the page or something
                }
            }
        });

        // Library
        spawn_local(library_thread(library, rx_a, tx_e, rx_down, rx_c, tx_up));

        let _ = leptos::mount::mount_to_body(move || render::body(reactive, tx_a));
    })());

    Ok(())
}

async fn library_thread(
    mut library: Library,
    mut rx_a: Receiver<Action>,
    mut tx_e: Sender<Event>,
    mut rx_down: Receiver<Message>,
    mut rx_c: Receiver<sync::ConnStatus>,
    mut tx_up: Sender<Message>,
) {
    let mut sync_state = automerge::sync::State::new();
    let mut connected = false;
    loop {
        select! {
           r_action = rx_a.recv() => {
                let action = r_action.context("rx_action").unwrap();
               let patches = library.apply(&action);
               disk::save_library(&mut library);
               for p in patches {
                   tx_e.send(p);
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
                            tx_e.send(p);
                        }
                        let our_heads = library.replicated.get_heads();
                        if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                            tx_up.send(Message::Library(message));
                        } else {
                            debug!( ?sync_state, ?our_heads, "no reply");
                        }
                    }

                    LibraryId(_id) => {
                        // TODO for now we only support one Library per user
                    }
                }
            },

            r_c = rx_c.recv() => {
                use sync::ConnStatus::*;
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
