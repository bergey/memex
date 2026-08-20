pub use crate::library::ids::{AuthToken, UserId};

use automerge::AutoCommit;

pub struct User {
    id: UserId,
    automerge: AutoCommit,
}
