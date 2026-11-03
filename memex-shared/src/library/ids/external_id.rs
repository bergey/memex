// serialize u128 as binary, but base64 in JSON

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{de, ser};
use std::fmt;

pub fn deserialize<'d, D>(d: D) -> Result<u128, D::Error>
where
  D: serde::Deserializer<'d>,
{
  if d.is_human_readable() {
    d.deserialize_string(ExternalIdDVisitor)
  } else {
    d.deserialize_u128(ExternalIdDVisitor)
  }
}

struct ExternalIdDVisitor;

impl<'d> de::Visitor<'d> for ExternalIdDVisitor {
  type Value = u128;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(
      formatter,
      "a better error message TODO"
    )
  }

  fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    let mut bytes = [0u8; 16];
    URL_SAFE_NO_PAD.decode_slice(s, &mut bytes).map_err(E::custom)?;
    let id = u128::from_le_bytes(bytes);
    Ok(id)
  }

  fn visit_u128<E>(self, input: u128) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(input)
  }
}

pub fn serialize<S>(id: &u128, s: S) -> Result<S::Ok, S::Error>
where
  S: ser::Serializer,
{
  if s.is_human_readable() {
    s.serialize_str(&URL_SAFE_NO_PAD.encode(id.to_le_bytes()))
  } else {
    s.serialize_u128(*id)
  }
}
