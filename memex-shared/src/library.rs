pub mod action;
mod crdt;
pub mod date;
pub mod ids;

use crate::errors::LogResult;
use action::*;
pub use ids::*;

use automerge::{
    self, ActorId, AutoCommit, ObjId, ObjId::Root, ObjType, ReadDoc, transaction::Transactable,
};
use tracing::info;

/// for now, it only makes sense to have one Library
/// in future, it will be possible for several users to share a Library, so useful for one User to have / belong to several Libraries
#[derive(Clone, Debug)]
pub struct Library {
    pub id: LibraryId,
    pub replicated: AutoCommit,
}

/// a Record may be a book, paper, movie, whatever
pub struct Record {
    pub id: RecordId,
    pub title: String,
    pub author: String,
    pub url: String,
    pub typ: String,
    pub date: date::Date,
    pub date_added: date::Date,
    pub read_last: date::Date,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
}

impl Library {
    pub fn new(actor_id: ActorId) -> Self {
        let mut am = AutoCommit::new();
        // Libraries created on separate clients should share the initial schema history
        // I'm not sure if this matters
        let nil_actor_id = ActorId::from(&[0u8; 8]);
        am.set_actor(nil_actor_id);
        am.put(Root, "name", "My Library").unwrap();
        am.put_object(Root, "records", ObjType::List).unwrap();
        am.set_actor(actor_id);
        am.update_diff_cursor(); // subscriber should never receive the changes above
        let id = LibraryId::random();
        info!(?id, "new library");
        Library { id, replicated: am }
    }

    pub fn name(&self) -> String {
        let (name, _) = self
            .replicated
            .get(Root, "name")
            .unwrap()
            .expect("library has name");
        name.into_string().expect("name is a string")
    }

    fn records_id(&self) -> ObjId {
        self.replicated
            .get(Root, "records")
            .unwrap()
            .expect("library has records")
            .1
    }

    pub fn records(&self) -> impl Iterator<Item = Record> {
        self.replicated
            .values(self.records_id())
            .map(|(_val, r_id)| Record {
                title: self.get_string(&r_id, "title"),
                author: self.get_string(&r_id, "author"),
                url: self.get_string(&r_id, "url"),
                typ: self.get_string(&r_id, "type"),
                date: self.get_date(&r_id, "date"),
                date_added: self.get_date(&r_id, "date_added"),
                read_last: self.get_date(&r_id, "read_last"),
                tags: self
                    .iter_set(&r_id, "tags")
                    .unwrap()
                    .map(|(val, t_id)| Tag {
                        id: TagId(t_id),
                        name: val.into_string().unwrap(),
                    })
                    .collect(),
                id: RecordId(r_id),
            })
    }

    pub fn apply(&mut self, action: &Action<()>) -> Vec<Event> {
        use action::Action::*;
        match action {
            SetName(name) => {
                self.replicated.put(Root, "name", name).unwrap();
            }

            AddRecord(()) => {
                let r_id = self.add_to_set(&self.records_id(), ObjType::Map);
                // consider hydrating from Record type
                self.replicated
                    .put(r_id.clone(), "title", "".to_string())
                    .unwrap();
                self.replicated.put(r_id, "author", "".to_string()).unwrap();
            }

            SetRecord(RecordId(r_id), field) => {
                use RecordField::*;
                use automerge::ScalarValue::Timestamp;
                let (prop, value): (&str, automerge::ScalarValue) = match field {
                    Title(v) => ("title", v.into()),
                    Author(v) => ("author", v.into()),
                    Url(v) => ("url", v.into()),
                    Type(v) => ("type", v.into()),
                    Date(v) => ("date", Timestamp(v.to_i64())),
                    DateAdded(v) => ("date_added", Timestamp(v.to_i64())),
                    ReadLast(v) => ("read_last", Timestamp(v.to_i64())),
                };
                self.replicated.put(r_id, prop, value).unwrap();
            }

            DeleteRecord(RecordId(item_id)) => {
                self.remove_from_set(&self.records_id(), item_id);
            }

            AddTag(RecordId(r_id), _) => {
                let tags = self.ensure_set(r_id, "tags").unwrap();
                let len = self.replicated.length(&tags);
                self.replicated.insert(&tags, len, "").unwrap();
            }

            DeleteTag(RecordId(r_id), TagId(t_id)) => {
                let tags = self.ensure_set(r_id, "tags").unwrap();
                self.remove_from_set(&tags, t_id);
            }

            SetTag(RecordId(r_id), TagId(t_id), s) => {
                let tags = self.ensure_set(r_id, "tags").unwrap();
                let o_index = self
                    .replicated
                    .values(&tags)
                    .position(|(_, id)| id == *t_id);
                if let Some(index) = o_index {
                    self.replicated.put(&tags, index, s.to_string()).unwrap();
                }
            }
        }

        self.get_patches()
    }

    pub fn get_patches(&mut self) -> Vec<Event> {
        self.replicated
            .diff_incremental()
            .into_iter()
            .filter_map(|p| Event::from_patch(p).log_error())
            .collect()
    }

    fn get_date(&self, r_id: &ObjId, field: &str) -> date::Date {
        self.get_i64(r_id, field)
            .map(date::Date::from_i64)
            .unwrap_or(Default::default())
    }
}
