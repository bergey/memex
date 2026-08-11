mod actor;
mod disk;
mod prelude;
mod render;
mod sync;

use automerge::sync::SyncDoc;
use prelude::*;
use memex_shared::library::{
    Library,
    action::{Action, Event},
};

use futures::channel::mpsc::Receiver;
use futures::select;
use std::panic;
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
    let (tx_up, rx_down) = if let Some(ws_url) = server_ws_url {
        sync::ServerSync::start(ws_url)
    } else {
        let (tx_up, _) = Sender::new(0);
        let (rx_up, rx_down) = Sender::new(0);
        std::mem::forget(rx_up);
        (tx_up, rx_down)
    };

    spawn_local((async move || {
        let library = disk::load_library("my_library").await;
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
        spawn_local(library_thread(library, rx_a, tx_e, rx_down, tx_up));

        let _ = leptos::mount::mount_to_body(move || render::body(reactive, tx_a));
    })());

    Ok(())
}

async fn library_thread(
    mut library: Library,
    mut rx_a: Receiver<Action>,
    mut tx_e: Sender<Event>,
    mut rx_down: Receiver<sync::Message>,
    mut tx_up: Sender<automerge::sync::Message>,
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
                       tx_up.send(message)
                   }
               }
           },
            r_down = rx_down.recv() => {
                use sync::Message::*;
                match r_down.context("rx_down").unwrap() {
                    Connected => connected = true,
                    Disconnected => connected = false,
                    Automerge(message) => {
                        library.replicated.sync().receive_sync_message(&mut sync_state, message).unwrap();
                        for p in library.get_patches(){
                            tx_e.send(p);
                        }
                        let our_heads = library.replicated.get_heads();
                        if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                            tx_up.send(message);
                        } else {
                            debug!( ?sync_state, ?our_heads, "no reply");
                        }
                    }
                }
            }
        }
    }
}
