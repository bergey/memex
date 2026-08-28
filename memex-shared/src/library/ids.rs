use automerge::ObjId;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use getrandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RecordId(pub ObjId);
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TagId(pub ObjId);
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AuthToken(u128);
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct UserId(u128);


#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LibraryId(pub u128);

// TODO this should probably be From / Into.  Anything left over should be a new Trait.
// maybe also some newtype deriving, unless the traits are already derivable
impl LibraryId {
    pub fn random() -> Self {
        let low = getrandom::u64().unwrap();
        let high = getrandom::u64().unwrap();
        Self::from_u64(high, low)
    }

    fn from_u64(high: u64, low: u64) -> Self {
        LibraryId(((high as u128) << 64) + low as u128)
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

impl UserId {
    pub fn random() -> Self {
        let low = getrandom::u64().unwrap();
        let high = getrandom::u64().unwrap();
        Self::from_u64(high, low)
    }

    fn from_u64(high: u64, low: u64) -> Self {
        UserId(((high as u128) << 64) + low as u128)
    }

    pub fn to_string(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0.to_le_bytes())
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let mut bytes = [0u8; 16];
        URL_SAFE_NO_PAD.decode_slice(s, &mut bytes).ok()?;
        let id = u128::from_le_bytes(bytes);
        Some(UserId(id))
    }

    pub fn to_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn from_uuid(uuid: &Uuid) -> Self {
        UserId(uuid.as_u128())
    }
}

impl AuthToken {
    pub fn random() -> Self {
        let low = getrandom::u64().unwrap();
        let high = getrandom::u64().unwrap();
        Self::from_u64(high, low)
    }

    fn from_u64(high: u64, low: u64) -> Self {
        AuthToken(((high as u128) << 64) + low as u128)
    }

    pub fn to_string(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0.to_le_bytes())
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let mut bytes = [0u8; 16];
        URL_SAFE_NO_PAD.decode_slice(s, &mut bytes).ok()?;
        let id = u128::from_le_bytes(bytes);
        Some(AuthToken(id))
    }

    pub fn to_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn from_uuid(uuid: &Uuid) -> Self {
        AuthToken(uuid.as_u128())
    }
}
