use automerge::sync as am;
use serde::{de, ser};
use std::fmt;

pub fn deserialize<'d, D>(d: D) -> Result<am::Message, D::Error>
where
    D: serde::Deserializer<'d>,
{
    // https://github.com/enarx/ciborium/issues/96 cannot use deserialize_bytes
    d.deserialize_byte_buf(AmMessageDVisitor)
}

struct AmMessageDVisitor;

impl<'d> de::Visitor<'d> for AmMessageDVisitor {
    type Value = am::Message;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a byte array produced by calling encode on an AM Message"
        )
    }

    // am::ReadMessageError
    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        am::Message::decode(bytes).map_err(|e| de::Error::custom(format!("automerge: {e}")))
    }
}

pub fn serialize<S>(message: &am::Message, s: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    s.serialize_bytes(message.clone().encode().as_ref())
}
