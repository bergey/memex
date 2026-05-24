use anyhow;
use wasm_bindgen::prelude::*;

pub struct Error(anyhow::Error);
pub type Result<A> = std::result::Result<A, Error>;

impl From<Error> for JsValue {
    fn from(Error(err): Error) -> Self {
        js_sys::Error::new(&std::format!("{}", err)).into()
    }
}
