use crate::prelude::*;
use memex_shared::{AuthToken, UserId};
use memex_shared::auth::AuthResponse;

use wasm_bindgen::prelude::*;

pub fn load_auth_token() -> Option<AuthToken> {
  let storage = local_storage()?;
  let s = storage.get_item("auth_token").log_error().flatten()?;
  AuthToken::from_str(s.as_ref())
}

pub fn save_auth_token(auth_token: AuthToken) {
  if let Some(storage) = local_storage() {
    storage
      .set_item("auth_token", &auth_token.to_string())
      .log_error();
  }
}

pub async fn signup_and_store() {
  if let Some(resp) = signup().await.log_error()
    && let Some(resp) = serde_wasm_bindgen::from_value::<AuthResponse>(resp).log_error()
  {
    save_auth_token(resp.auth_token);
    save_user_id(resp.user_id);
  }
}

pub fn load_user_id() -> Option<UserId> {
  let storage = local_storage()?;
  let s = storage.get_item("user_id").log_error().flatten()?;
  UserId::from_str(s.as_ref())
}

pub fn save_user_id(user_id: UserId) {
  if let Some(storage) = local_storage() {
    storage
      .set_item("user_id", &user_id.to_string())
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
  pub async fn signup() -> std::result::Result<JsValue, JsValue>;
}
