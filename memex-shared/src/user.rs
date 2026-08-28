pub use crate::library::ids::{AuthToken, UserId};

use automerge::AutoCommit;

#[allow(dead_code)]
pub struct User {
    id: UserId,
    automerge: AutoCommit,
}
