use automerge::ObjId;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use getrandom;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RecordId(pub ObjId);
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TagId(pub ObjId);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct LibraryId(pub u64);

impl LibraryId {
    pub fn random() -> Self {
        LibraryId(getrandom::u64().unwrap())

    }

    pub fn to_string(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0.to_le_bytes())
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let mut bytes = [0u8; 8];
        URL_SAFE_NO_PAD.decode_slice(s, &mut bytes).ok()?;
        let id = u64::from_le_bytes(bytes);
        Some(LibraryId(id))
    }
}
