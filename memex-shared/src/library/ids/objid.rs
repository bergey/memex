use automerge::ObjId;
use serde::{de, ser};
use std::fmt;

pub fn deserialize<'d, D>(d: D) -> Result<ObjId, D::Error>
where
  D: serde::Deserializer<'d>,
{
  // https://github.com/enarx/ciborium/issues/96 cannot use deserialize_bytes
  d.deserialize_byte_buf(AmIdDVisitor)
}

struct AmIdDVisitor;

impl<'d> de::Visitor<'d> for AmIdDVisitor {
  type Value = ObjId;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(
      formatter,
      "a byte array produced by calling encode on an AM ObjId"
    )
  }

  fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    ObjId::try_from(bytes).map_err(|e| de::Error::custom(e))
  }
}

pub fn serialize<S>(id: &ObjId, s: S) -> Result<S::Ok, S::Error>
where
  S: ser::Serializer,
{
  s.serialize_bytes(id.to_bytes().as_ref())
}
