use automerge::sync as am;

use crate::library::LibraryId;

#[derive(Debug, Clone)]
pub enum Message {
    Library(am::Message),
    LibraryId(LibraryId),
    // TODO future: user, sharing
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        // TODO derive
        vec![]
    }

    pub fn decode(_bytes: &[u8]) -> anyhow::Result<Self> {
        // TODO derive
        Err(anyhow::anyhow!("not implemented"))
    }
}
