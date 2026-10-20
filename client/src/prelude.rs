pub use anyhow::Context;
#[allow(unused_imports)]
pub use anyhow::anyhow;
use futures::channel::mpsc;
pub use memex_shared::errors::*;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
#[allow(unused_imports)]
pub use tracing::{debug, error, info, warn};
use wasm_bindgen::prelude::*;
use web_sys::Storage;

#[derive(Debug)]
pub struct MemexError(anyhow::Error);
pub type Result<A, E = MemexError> = std::result::Result<A, E>;

impl Display for MemexError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    Debug::fmt(&self.0, formatter)
  }
}

impl From<MemexError> for JsValue {
  fn from(MemexError(err): MemexError) -> Self {
    js_sys::Error::new(&std::format!("{}", err)).into()
  }
}

impl<T: Into<anyhow::Error>> From<T> for MemexError {
  fn from(err: T) -> Self {
    MemexError(err.into())
  }
}

#[derive(Clone, Debug)]
pub struct Sender<T>(mpsc::Sender<T>);

impl<T: Debug> From<mpsc::Sender<T>> for Sender<T> {
  fn from(inner: mpsc::Sender<T>) -> Self {
    Sender(inner)
  }
}

impl<T: Debug> Sender<T> {
  pub fn new(capacity: usize) -> (Self, mpsc::Receiver<T>) {
    let (tx, rx) = mpsc::channel(capacity);
    (Sender(tx), rx)
  }

  pub fn send(&mut self, value: T) {
    match self.0.try_send(value) {
      Ok(_) => {}
      Err(cause) => warn!(?cause, "failed to enqueue message"),
    }
  }
}

#[derive(Clone, Debug)]
pub enum ToWorker<T> {
  Broadcast(web_sys::BroadcastChannel, PhantomData<T>),
  Port(web_sys::MessagePort, PhantomData<T>),
}

impl<T: Serialize> ToWorker<T> {
  pub fn broadcast(name: &str) -> Self {
    let broadcast = web_sys::BroadcastChannel::new(name)
      .log_error()
      .unwrap();
    Self::Broadcast(broadcast, PhantomData)
  }

  pub fn port(port: web_sys::MessagePort) -> Self {
    Self::Port(port, PhantomData)
  }

  pub fn send(&mut self, t: &T) {
    if let Some(js_value) = serde_wasm_bindgen::to_value(t).log_error() {
      match self {
        Self::Broadcast(b, _) => b.post_message(&js_value).log_error(),
        Self::Port(p, _) => p.post_message(&js_value).log_error(),
      };
    }
  }
}

pub fn local_storage() -> Option<Storage> {
  web_sys::window().and_then(|w| w.local_storage().log_error().flatten())
}

pub fn setup_observability() {
  console_error_panic_hook::set_once();
  wasm_tracing::set_as_global_default();
}
