mod automerge;

use crate::library::LibraryId;
use crate::user::AuthToken;

use ::automerge::sync as am;
use ciborium;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    #[serde(rename = "l")]
    #[serde(with = "automerge")]
    Library(am::Message),
    #[serde(rename = "li")]
    LibraryId(LibraryId),
    Authorize(AuthToken),
    // TODO future: user, sharing
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        let _ = ciborium::into_writer(self, &mut vec);
        vec
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let msg = ciborium::from_reader(bytes)?;
        Ok(msg)
    }
}
