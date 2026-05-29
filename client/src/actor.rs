use crate::prelude::*;

use automerge::ActorId;
use std::str::FromStr;

// load stored actor or make one
pub fn local_actor_id() -> Result<ActorId> {
    let o_storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    match o_storage {
        None => Ok(ActorId::random()), // no storage,
        Some(storage) => {
            if let Some(s) = storage.get_item("actor_id").unwrap() {
                if let Ok(id) = ActorId::from_str(&s) {
                    return Ok(id);
                }
            }
            let id = ActorId::random();
            storage.set_item("actor_id", &id.to_hex_string()).unwrap();
            Ok(id)
        }
    }
}
