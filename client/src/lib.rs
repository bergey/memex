mod library;
mod prelude;
mod render;

use prelude::*;

use log::Level;
use std::panic;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::Info;

#[wasm_bindgen]
pub fn start(_server_ws_url: Option<String>) -> Result<()> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(LOG_LEVEL).expect("failed to init logging");

    // TODO load or create library
    let library = library::Library::new();
    // TODO load or create actor ID
    // let actor_id = automerge::ActorId::random();
    // library.set_actor(actor_id);
    let library_ref = Arc::new(Mutex::new(library));

    let _ = leptos::mount::mount_to_body(move || render::body(library_ref));
    Ok(())
}
