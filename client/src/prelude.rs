#[allow(unused_imports)]
pub use anyhow::anyhow;
#[allow(unused_imports)]
pub use tracing::{debug, error, info, warn};
use std::fmt::{Debug, Display};
use wasm_bindgen::prelude::*;
use futures::channel::mpsc;
pub use memex_shared::errors::*;

#[derive(Debug)]
pub struct MemexError(anyhow::Error);
pub type Result<A, E=MemexError> = std::result::Result<A, E>;

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

impl<T: Debug> Sender<T> {
    pub fn new(inner: mpsc::Sender<T>) -> Self {
        Sender(inner)
    }

    pub fn send(&mut self, value: T) {
        match self.0.try_send(value) {
            Ok(_) => {},
            Err(e) => warn!("failed to enqueue message: {}", e),
        }
    }
}
