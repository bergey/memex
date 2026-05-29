pub mod action;

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
// TODO &str instead of owned Strings
pub struct Record {
    pub id: RecordId,
    pub title: String,
    pub author: String,
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
        Library {
            replicated,
        }
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
                let (title, _) = self
                    .replicated
                    .get(&r_id, "title")
                    .log_error()
                    .flatten()
                    .expect("record has title");
                let author = self
                    .replicated
                    .get(&r_id, "author")
                    .log_error()
                    .flatten()
                    .map(|a| a.0.into_string().ok())
                    .flatten()
                    .unwrap_or_else(|| "".to_string());

                Record {
                    id: RecordId(r_id),
                    title: title.into_string().expect("title is a string"),
                    author: author,
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

            SetTitle(RecordId(r_id), title) => {
                self.replicated.put(r_id, "title", title).unwrap();
            }

            SetAuthor(RecordId(r_id), author) => {
                self.replicated.put(r_id, "author", author).unwrap();
            }

            DeleteRecord(RecordId(item_id)) => {
                self.remove_from_set(&self.records_id(), item_id);
            }
        }

        self.replicated
            .diff_incremental()
            .into_iter()
            .filter_map(|p| Event::from_patch(p).log_error())
            .collect()
    }
}
