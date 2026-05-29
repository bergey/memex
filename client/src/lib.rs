mod library;
mod prelude;
mod render;

use crate::library::action::{Action, Event};
use prelude::*;

use log::Level;
use std::panic;
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::Info;

#[wasm_bindgen]
pub fn start(_server_ws_url: Option<String>) -> Result<()> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(LOG_LEVEL).expect("failed to init logging");

    let (tx_a, rx_a) = mpsc::sync_channel::<Action>(3);
    let (tx_e, rx_e) = mpsc::sync_channel::<Event>(3);
    let mut tx_e = Sender::new(tx_e);

    spawn_local((async move || {
        let mut library = library::Library::load("my_library").await;
        let reactive = render::ReactiveLibrary::from_replicated(&library);
        let mut reactive_for_updates = reactive.clone();

        spawn_local(async move {
            loop {
                match rx_e.recv() {
                    Ok(action) => reactive_for_updates.apply(action),
                    Err(_) => break,
                }
            }
        });

        spawn_local(async move {
            loop {
                match rx_a.recv() {
                    Ok(event) => {
                        let patches = library.apply(&event);
                        for p in patches {
                            tx_e.send(p);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let _ =
            leptos::mount::mount_to_body(move || render::body(reactive, Sender::new(tx_a)));
    })());

    Ok(())
}
