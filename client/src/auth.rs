use crate::disk::open_database;
use crate::prelude::*;
use crate::sync::ToWorker;
use memex_shared::auth::AuthResponse;
use memex_shared::{AuthToken, UserId};

use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
use wasm_bindgen::prelude::*;

pub async fn load_auth_token() -> Option<AuthToken> {
  let db = open_database().await.log_error()?;
  let tx = db
    .transaction("account")
    .with_mode(TransactionMode::Readonly)
    .build()
    .log_error()?;
  let store = tx.object_store("account").log_error()?;
  let token = store.get("auth_token").serde().log_error()?.await.log_error()?;
  tx.commit().await.log_error()?;
  token
}

pub async fn save_auth_token(auth_token: AuthToken) -> Option<()> {
  let db = open_database().await.log_error()?;
  let tx = db
    .transaction("account")
    .with_mode(TransactionMode::Readwrite)
    .build()
    .log_error()?;
  let store = tx.object_store("account").log_error()?;
  store.put(auth_token).with_key("auth_token".to_string()).serde().log_error()?.await.log_error()?;
  tx.commit().await.log_error()?;
  Some(())
}

pub async fn signup_and_store(mut tx: BrowserPort<ToWorker>) {
  if let Some(resp) = signup().await.log_error()
    && let Some(resp) = serde_wasm_bindgen::from_value::<AuthResponse>(resp).log_error()
  {
    save_user_id(resp.user_id);
    debug!("saved auth token, trying to connect WS");
    tx.send(&ToWorker::Authenticate(resp.auth_token))
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
