use crate::prelude::*;
use memex_shared::AuthToken;

use wasm_bindgen::prelude::*;

pub fn load_auth_token() -> Option<AuthToken> {
  let storage = local_storage()?;
  let s = storage.get_item("auth_token").log_error().flatten()?;
  AuthToken::from_str(s.as_ref())
}

pub fn save_auth_token(auth_token: AuthToken) {
  if let Some(storage) = local_storage() {
    let _ = storage
      .set_item("auth_token", &auth_token.to_string())
      .log_error();
  }
}

// where should User ID / name come from?
#[allow(dead_code)]
pub fn request_auth_token() -> Result<AuthToken> {
  panic!("not implemented");
}

#[wasm_bindgen(module = "/src/auth.js")]
extern "C" {
  #[wasm_bindgen(catch)]
  pub async fn signup() -> std::result::Result<String, JsValue>;
}
