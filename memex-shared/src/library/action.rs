use super::date;
use super::{RecordId, TagId};

use tracing::*;

use anyhow::{Result, anyhow};

// add is () when starting an edit, (usize, ObjId) when finishing
#[derive(Clone, Debug)]
pub enum Action<AR = (), DR = RecordId, AT = (), DT = TagId> {
    SetName(String),
    AddRecord(AR),
    SetRecord(RecordId, RecordField),
    DeleteRecord(DR),
    AddTag(RecordId, AT),
    SetTag(RecordId, TagId, String),
    DeleteTag(RecordId, DT),
}

#[derive(Clone, Debug)]
pub enum RecordField {
    Title(String),
    Author(String),
    Url(String),
    Type(String), // TODO enum?
    Date(date::Date),
    DateAdded(date::Date),
    ReadLast(date::Date),
}
pub type Event = Action<(usize, RecordId), usize, TagId, usize>;

fn date_from(i: i64) -> date::Date {
    date::Date::from_i64(i)
}

impl Event {
    pub fn from_patch(patch: automerge::patches::Patch) -> Result<Self> {
        use Action::*;
        use RecordField::*;
        use automerge::patches::PatchAction::*;
        debug!("{:?}", patch);
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
                            ("date", Some(i), _) => Some(RecordField::Date(date_from(i))),
                            ("date_added", Some(i), _) => Some(DateAdded(date_from(i))),
                            ("read_last", Some(i), _) => Some(ReadLast(date_from(i))),
                            _ => None,
                        };
                        match o_field {
                            Some(field) => Ok(SetRecord(RecordId(patch.obj), field)),
                            None => Err(anyhow!("unknown action on a record"))?,
                        }
                    }
                    (1, DeleteSeq { index, length: 1 }) => Ok(DeleteRecord(index)),
                    (1, Insert { index, values }) if values.len() == 1 => Ok(AddRecord((
                        index,
                        RecordId(values.iter().next().unwrap().1.clone()),
                    ))),
                    (3, Insert {..}) => Ok(AddTag(RecordId(patch.path.last().unwrap().0.clone()), TagId(patch.obj))),

                    // Patch { obj: Id(46, ActorID("10797249f520da82170bf3767d2ba329"), 3), path: [(Root, Map("records")), (Id(2, ActorID("095ddb9eff479f5cc9968ff5e7190c04"), 1), Seq(0)), (Id(3, ActorID("095ddb9eff479f5cc9968ff5e7190c04"), 1), Map("tags"))], action: PutSeq { index: 6, value: (Scalar(Str("g")), Id(67, ActorID("a5b41846a13d48c95b54555af8879f01"), 9)), conflict: false } }
                    (3, PutSeq { value, ..}) => Ok(SetTag(RecordId(patch.path.last().unwrap().0.clone()), TagId(patch.obj), value.0.into_string().unwrap())),

                    // Patch { obj: Id(46, ActorID("10797249f520da82170bf3767d2ba329"), 3), path: [(Root, Map("records")), (Id(2, ActorID("095ddb9eff479f5cc9968ff5e7190c04"), 1), Seq(0)), (Id(3, ActorID("095ddb9eff479f5cc9968ff5e7190c04"), 1), Map("tags"))], action: DeleteSeq { index: 8, length: 1 } }
                    (3, DeleteSeq {index, ..}) => Ok(DeleteTag(RecordId(patch.path.last().unwrap().0.clone()), index)),

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
