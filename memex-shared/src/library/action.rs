use super::RecordId;

use anyhow::{Result, anyhow};

// add is () when starting an edit, (usize, ObjId) when finishing
#[derive(Clone, Debug)]
pub enum Action<AR = (), DR = RecordId> {
    SetName(String),
    AddRecord(AR),
    SetRecord(RecordId, RecordField),
    DeleteRecord(DR),
}

#[derive(Clone, Debug)]
pub enum RecordField {
    Title(String),
    Author(String),
    Url(String),
    Type(String), // TODO enum?
    Date(i64), // seconds since 1970
    DateAdded(i64),
    ReadLast(i64),
}
pub type Event = Action<(usize, RecordId), usize>;

impl Event {
    pub fn from_patch(patch: automerge::patches::Patch) -> Result<Self> {
        use Action::*;
        use RecordField::*;
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
                    ) => {
                        let o_field = match (key.as_ref(), value.to_i64(), value.into_string()) {
                            ("title", _, Ok(s)) => Some(Title(s)),
                            ("author", _, Ok(s)) => Some(Author(s)),
                            ("url", _, Ok(s)) => Some(Url(s)),
                            ("type", _, Ok(s)) => Some(Type(s)),
                            ("date", Some(i), _) => Some(Date(i)),
                            ("date_added", Some(i), _) => Some(DateAdded(i)),
                            ("read_last", Some(i), _)  => Some(ReadLast(i)),
                            _ => None
                        };
                        match o_field {
                            Some(field) => Ok(SetRecord(RecordId(patch.obj), field)),
                            None => Err(anyhow!("unknown action on a record"))?,
                        }
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
    use RecordField::*;
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
        lib.apply(&SetRecord(first.id, Title(t.clone())));
        let after = lib.records().next().unwrap();
        assert_eq!(after.title, t);
    }

    fn adds_and_deletes() -> impl Strategy<Value = Action<(), usize>> {
        prop_oneof![
            // For cases without data, `Just` is all you need
            Just(AddRecord(())),
            // For cases with data, write a strategy for the interior data, then
            // map into the actual enum case.
            any::<usize>().prop_map(DeleteRecord)
        ]
    }

    proptest! {
            #[test]
            fn set_any_name(n in "\\PC*") {
              let mut lib = Library::new(ActorId::random());
              lib.apply(&SetName(n.clone()));
              assert_eq!(lib.name(), n);
            }

            #[test]
            fn delete_any_record(n in 1..=10usize, i in 0..10000usize) {
                // use a large range for i so that the propability of i % n is nearly uniform
                // without discarding test runs
                let mut lib = Library::new(ActorId::random());
                for _ in 0..n {
                    lib.apply(&AddRecord(()));
                }
                let record_id = lib.records().map(|r| r.id).nth(i % n).unwrap();
                lib.apply(&DeleteRecord(record_id));
            }

        #[test]
        fn adds_and_deletes_seq(actions in prop::collection::vec(adds_and_deletes(), 1..30)) {
            let mut lib = Library::new(ActorId::random());
            for a in actions {
                let record_ids = lib.records().map(|r| r.id).collect::<Vec<_>>();
                let act = match a {
                    AddRecord(()) => AddRecord(()), // technically a different type
                    DeleteRecord(i) => if record_ids.len() == 0 {
                        AddRecord(())
                    } else {
                        DeleteRecord(record_ids[i % record_ids.len()].clone())
                    },
                    _ => panic!("not reachable with this generator"),
                };
                let _ = lib.apply(&act);
            }
        }
    }
}
