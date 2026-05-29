pub use anyhow::anyhow;
#[allow(unused_imports)]
pub use log::{debug, error, info, warn};
use std::fmt::{Debug, Display};
use wasm_bindgen::prelude::*;
use std::sync::mpsc;

#[derive(Debug)]
pub struct MemexError(anyhow::Error);
pub type Result<A, E=MemexError> = std::result::Result<A, E>;

impl Display for MemexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Debug::fmt(&self.0, formatter)
    }
}

pub trait LogResult<A> {
    fn log_error(self) -> Option<A>;
}

impl<A, E: Debug> LogResult<A> for std::result::Result<A, E> {
    fn log_error(self) -> Option<A> {
        match self {
            Ok(a) => Some(a),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
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
pub struct Sender<T>(mpsc::SyncSender<T>);

impl<T: Debug> Sender<T> {
    pub fn new(inner: mpsc::SyncSender<T>) -> Self {
        Sender(inner)
    }

    pub fn send(&mut self, value: T) {
        match self.0.try_send(value) {
            Ok(_) => {},
            Err(mpsc::TrySendError::Full(val)) => warn!("failed to enqueue message: {:?}", val),
            Err(mpsc::TrySendError::Disconnected(_)) => error!("queue is disconnected, reload the page")
        }
    }
}
