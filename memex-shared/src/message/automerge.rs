use automerge::sync;
use serde::{de, ser};
use std::fmt;

pub fn deserialize<'d, D>(d: D) -> Result<sync::Message, D::Error>
where
    D: serde::Deserializer<'d>,
{
    // https://github.com/enarx/ciborium/issues/96 cannot use deserialize_bytes
    d.deserialize_byte_buf(AmMessageDVisitor)
}

struct AmMessageDVisitor;

impl<'d> de::Visitor<'d> for AmMessageDVisitor {
    type Value = sync::Message;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a byte array produced by calling encode on an AM Message"
        )
    }

    // sync::ReadMessageError
    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        sync::Message::decode(bytes).map_err(|e| de::Error::custom(e))
    }
}

pub fn serialize<S>(message: &sync::Message, s: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    s.serialize_bytes(message.clone().encode().as_ref())
}
