pub mod action;
pub mod date;

use crate::errors::LogResult;
use action::*;

use automerge::{
    self, ActorId, AutoCommit, ObjId, ObjId::Root, ObjType, ReadDoc, transaction::Transactable,
};

/// for now, it only makes sense to have one Library
/// in future, it will be possible for several users to share a Library, so useful for one User to have / belong to several Libraries
pub struct Library {
    pub replicated: AutoCommit,
}

/// a Record may be a book, paper, movie, whatever
pub struct Record {
    pub id: RecordId,
    pub title: String,
    pub author: String,
    pub url: String,
    pub typ: String,
    pub date: Option<date::Date>,
    pub date_added: Option<date::Date>,
    pub read_last: Option<date::Date>,
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RecordId(ObjId);

impl Library {
    pub fn new(actor_id: ActorId) -> Self {
        let mut am = AutoCommit::new();
        am.set_actor(actor_id);
        am.put(Root, "name", "My Library").unwrap();
        am.put_object(Root, "records", ObjType::List).unwrap();
        am.update_diff_cursor(); // subscriber should never receive the changes above
        Self::from_replicated(am)
    }

    pub fn from_replicated(replicated: AutoCommit) -> Self {
        Library { replicated }
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
            .map(|(_val, r_id)| {
                Record {
                    title: self.get_string(&r_id, "title"),
                    author: self.get_string(&r_id, "author"),
                    url: self.get_string(&r_id, "url"),
                    typ: self.get_string(&r_id, "type"),
                    date: self.get_i64(&r_id, "date").map(date::Date::from_i64),
                    date_added: self.get_i64(&r_id, "date_added").map(date::Date::from_i64),
                    read_last: self.get_i64(&r_id, "read_last").map(date::Date::from_i64),
                    id: RecordId(r_id),
                }
            })
    }

    fn add_to_set(&mut self, set_id: &ObjId, obj_type: ObjType) -> ObjId {
        let len = self.replicated.length(set_id);
        self.replicated
            .insert_object(set_id, len, obj_type)
            .unwrap()
    }

    fn remove_from_set(&mut self, set_id: &ObjId, item_id: &ObjId) {
        let o_index = self
            .replicated
            .values(set_id)
            .position(|(_, id)| id == *item_id);
        if let Some(index) = o_index {
            self.replicated
                .splice(set_id, index, 1, std::iter::empty::<&str>())
                .unwrap();
        }
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

    fn get_string(&self, r_id: &ObjId, field: &str) -> String {
        self.replicated
            .get(&r_id, field)
            .log_error()
            .flatten()
            .map(|a| a.0.into_string().ok())
            .flatten()
            .unwrap_or_else(|| "".to_string())
    }

    fn get_i64(&self, r_id: &ObjId, field: &str) -> Option<i64> {
        self.replicated
            .get(&r_id, field)
            .log_error()
            .flatten()
            .and_then(|a| a.0.to_i64())
    }
}
