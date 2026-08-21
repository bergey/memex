use crate::prelude::*;
use memex_shared::AuthToken;

use std::str::FromStr;

pub fn load_auth_token() -> Option<AuthToken> {
    let storage = local_storage()?;
    let s = storage.get_item("auth_token").log_error().flatten()?;
    AuthToken::from_str(&s)

}

pub fn save_auth_token(_auth_token: AuthToken) {
    panic!("not implemented");
}

// where should User ID / name come from?
pub fn request_auth_token() -> Result<AuthToken> {
    panic!("not implemented");
}
