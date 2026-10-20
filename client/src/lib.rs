mod actor;
mod auth;
mod disk;
mod prelude;
mod render;
mod sync;

use prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn start(worker_port: web_sys::MessagePort) -> Result<()> {
  setup_observability();

  spawn_local(async move {
    // TODO send library from worker thread?
    let library = disk::load_some_library().await;
    let reactive = render::ReactiveLibrary::from_replicated(&library);

    if let Some(broadcast) = web_sys::BroadcastChannel::new("shared_worker").log_error() {
      let reactive = reactive.clone();
      let cb = broadcast_callback(reactive);
      broadcast.set_onmessage(Some(cb.as_ref().unchecked_ref()));
      cb.forget();
    }

    let to_worker = ToWorker::port(worker_port);
    let _ = leptos::mount::mount_to_body(move || render::body(reactive, to_worker));
  });

  Ok(())
}

// DOM updates
fn broadcast_callback(mut reactive: render::ReactiveLibrary) -> Closure<dyn FnMut(web_sys::MessageEvent)> {
  Closure::new(move |ev: web_sys::MessageEvent| {
    if let Some(event) = serde_wasm_bindgen::from_value(ev.data()).log_error() {
      reactive.apply(event);
    }
  })
}
