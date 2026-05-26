pub use anyhow::anyhow;
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
pub use log::{debug, info, warn, error};

#[derive(Debug)]
pub struct Error(anyhow::Error);
pub type Result<A> = std::result::Result<A, Error>;

impl From<Error> for JsValue {
    fn from(Error(err): Error) -> Self {
        js_sys::Error::new(&std::format!("{}", err)).into()
    }
}

impl<T: Into<anyhow::Error>> From<T> for Error {
    fn from(err: T) -> Self {
        Error(err.into())
    }
}
