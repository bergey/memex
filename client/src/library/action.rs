use super::RecordId;
use crate::prelude::*;

// add is () when starting an edit, (usize, ObjId) when finishing
#[derive(Debug)]
pub enum Action<AR = (), DR = RecordId> {
    SetName(String),
    AddRecord(AR),
    SetTitle(RecordId, String),
    DeleteRecord(DR), // TODO move index lookup into Library::apply
}
pub type Event = Action<(usize, RecordId), usize>;

impl Event {
    pub fn from_patch(patch: automerge::patches::Patch) -> Result<Self> {
        use automerge::patches::PatchAction::*;
        use Action::*;
        if patch.path.len() == 0 {
            match patch.action {
                PutMap {
                    key,
                    value: (value, _),
                    ..
                } if key == "name" => Ok(SetName(value.to_string())),
                _ => Err(anyhow!("unknown action on root of library"))?,
            }
        } else {
            match &patch.path[0].1 {
                automerge::Prop::Map(k) if k == "records" => match (patch.path.len(), patch.action)
                {
                    (
                        2,
                        PutMap {
                            key,
                            value: (value, _),
                            ..
                        },
                    ) => match (key.as_ref(), value.into_string()) {
                        ("title", Ok(s)) => Ok(SetTitle(RecordId(patch.obj), s)),
                        _ => Err(anyhow!("unknown action on a record"))?,
                    },
                    (1, DeleteSeq { index, length: 1 }) => Ok(DeleteRecord(index)),
                    (1, Insert { index, values }) if values.len() == 1 => Ok(AddRecord((
                        index,
                        RecordId(values.iter().next().unwrap().1.clone()),
                    ))),
                    _ => Err(anyhow!("unknown action on records"))?,
                },
                _ => Err(anyhow!("unknown key on root of library"))?,
            }
        }
    }
}
