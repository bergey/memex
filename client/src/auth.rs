use crate::prelude::*;
use memex_shared::AuthToken;

use wasm_bindgen::prelude::*;

pub fn load_auth_token() -> Option<AuthToken> {
    let storage = local_storage()?;
    let s = storage
        .get_item("auth_token")
        .log_error("get auth_token")
        .flatten()?;
    AuthToken::from_str(s.as_ref())
}

#[allow(dead_code)]
pub fn save_auth_token(auth_token: AuthToken) {
    match local_storage() {
        None => warn!("unable to save auth_token"),
        Some(storage) => {
            let _ = storage
                .set_item("auth_token", &auth_token.to_string())
                .log_error("set auth_token");
        }
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
