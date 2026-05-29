use super::RecordId;
use anyhow::{Result, anyhow};

// add is () when starting an edit, (usize, ObjId) when finishing
#[derive(Clone, Debug)]
pub enum Action<AR = (), DR = RecordId> {
    SetName(String),
    AddRecord(AR),
    SetTitle(RecordId, String),
    SetAuthor(RecordId, String),
    DeleteRecord(DR),
}
pub type Event = Action<(usize, RecordId), usize>;

impl Event {
    pub fn from_patch(patch: automerge::patches::Patch) -> Result<Self> {
        use Action::*;
        use automerge::patches::PatchAction::*;
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
                        ("author", Ok(s)) => Ok(SetAuthor(RecordId(patch.obj), s)),
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

// ∀ l : Library, a : Action . ∃ e : Event . from_patch(l.apply(a)) = Ok(e)
// ∀ as : [Action] . fold Library::new as (λ l a. l.apply(a)) = l ==> l.records() does not crash
// l.name does not crash, &c

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::library::Library;
    use Action::*;
    use automerge::ActorId;
    use proptest::prelude::*;

    #[test]
    fn set_name() {
        let n = "my name".to_string();
        let mut lib = Library::new(ActorId::random());
        lib.apply(&SetName(n.clone()));
        assert_eq!(lib.name(), n);
    }

    #[test]
    fn add_record_len() {
        let mut lib = Library::new(ActorId::random());
        lib.apply(&AddRecord(()));
        assert_eq!(lib.records().collect::<Vec<_>>().len(), 1);
    }

    #[test]
    fn set_title() {
        let t = "my title".to_string();
        let mut lib = Library::new(ActorId::random());
        lib.apply(&AddRecord(()));
        let first = lib.records().next().unwrap();
        lib.apply(&SetTitle(first.id, t.clone()));
        let after = lib.records().next().unwrap();
        assert_eq!(after.title, t);
    }

    proptest! {
        #[test]
        fn set_any_name(n in "\\PC*") {
          let mut lib = Library::new(ActorId::random());
          lib.apply(&SetName(n.clone()));
          assert_eq!(lib.name(), n);
        }
    }
}
