mod library;
mod prelude;
mod render;

use prelude::*;

use log::{Level, info};
use std::panic;
use wasm_bindgen::{prelude::*};

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::Info;

#[wasm_bindgen]
pub fn start(_server_ws_url: Option<String>) -> Result<()> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(LOG_LEVEL).expect("failed to init logging");

    info!("entered rust via webassembly");

    let _ = leptos::mount::mount_to_body(render::body);
    Ok(())
}
