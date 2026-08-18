use automerge::ObjId;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use getrandom;
use uuid::Uuid;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RecordId(pub ObjId);
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TagId(pub ObjId);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct LibraryId(pub u128);

impl LibraryId {
    pub fn random() -> Self {
        let low = getrandom::u64().unwrap();
        let high = getrandom::u64().unwrap();
        LibraryId((high as u128) << 64 + low as u128)

    }

    pub fn to_string(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0.to_le_bytes())
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let mut bytes = [0u8; 16];
        URL_SAFE_NO_PAD.decode_slice(s, &mut bytes).ok()?;
        let id = u128::from_le_bytes(bytes);
        Some(LibraryId(id))
    }

    pub fn to_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn from_uuid(uuid: &Uuid) -> Self {
        LibraryId(uuid.as_u128())
    }
}
