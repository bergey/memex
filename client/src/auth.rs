use crate::prelude::*;
use memex_shared::AuthToken;

use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys as browser;
// use webauthn_rs_proto::attest as server;

pub fn load_auth_token() -> Option<AuthToken> {
    let storage = local_storage()?;
    let s = storage.get_item("auth_token").log_error().flatten()?;
    let uuid = Uuid::from_str(&s).ok()?; // TODO store in base-64 locally
    Some(AuthToken::from_uuid(&uuid))
}

pub fn save_auth_token(_auth_token: AuthToken) {
    panic!("not implemented");
}

// where should User ID / name come from?
pub fn request_auth_token() -> Result<AuthToken> {
    panic!("not implemented");
}

fn credentials() -> Option<browser::CredentialsContainer> {
    web_sys::window().map(|window| window.navigator().credentials())
}

#[wasm_bindgen(module = "/src/auth.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn signup() -> std::result::Result<String, JsValue>;
}
