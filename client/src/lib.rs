mod actor;
mod disk;
mod prelude;
mod render;
mod sync;

use automerge::sync as am;
use automerge::sync::SyncDoc;
use memex_shared::library::action::{Action, Event};
use prelude::*;

use futures::{channel::mpsc, select};
use log::Level;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::Info;

#[wasm_bindgen]
pub fn start(server_ws_url: Option<String>) -> Result<()> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(LOG_LEVEL).expect("failed to init logging");

    let (tx_a, mut rx_a) = Sender::new(3);
    let (mut tx_e, mut rx_e) = Sender::new(3);
    let (mut tx_up, mut rx_up) = Sender::new(3);
    let (tx_down, mut rx_down) = Sender::new(10);

    spawn_local((async move || {
        let mut library = disk::load_library("my_library").await;
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
        spawn_local(async move {
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
                       if let Some(message) = library.replicated.sync().generate_sync_message(&mut sync_state) {
                           tx_up.send(message)
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
                            }
                        }
                    }
                }
            }
        });

        // Network
        if let Some(ws_url) = server_ws_url {
            let mut net = sync::ServerSync::new(&ws_url, tx_down);
            spawn_local(async move {
                net.connect();
                loop {
                    let msg = rx_up.recv().await.expect("_up channel closed");
                    net.send(msg);
                }
            });
        }

        let _ = leptos::mount::mount_to_body(move || render::body(reactive, tx_a));
    })());

    Ok(())
}
