use crate::prelude::*;

use automerge::ActorId;
use std::str::FromStr;

// load stored actor or make one
pub fn local_actor_id() -> ActorId {
    match local_storage() {
        None => ActorId::random(), // no storage,
        Some(storage) => {
            if let Some(id) = storage.get_item("actor_id").log_error().flatten().and_then(|s| ActorId::from_str(&s).log_error()) {
                return id;
            }
            let id = ActorId::random();
            let _ = storage.set_item("actor_id", &id.to_hex_string()).log_error();
            id
        }
    }
}
