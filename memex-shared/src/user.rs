use automerge::AutoCommit;

pub struct UserId(u128);

pub struct User {
    id: UserId,
    automerge: AutoCommit,
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
