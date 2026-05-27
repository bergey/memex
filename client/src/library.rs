pub mod action;

use action::*;
// use crate::prelude::*;

use automerge::{self, AutoCommit, ReadDoc, ObjId, ObjId::Root, ObjType, transaction::Transactable};
use std::sync::{Arc, Mutex};

/// for now, it only makes sense to have one Library
/// in future, it will be possible for several users to share a Library, so useful for one User to have / belong to several Libraries
pub struct Library {
    replicated: AutoCommit,
    pub subscriber: Box<dyn FnMut(Event) + Send>,
    counter: u64, // temporary hack to distinguish new records
}

pub type LibraryRef = Arc<Mutex<Library>>;

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
    pub fn new() -> Self {
        let mut am = AutoCommit::new();
        am.put(Root, "name", "My Library").unwrap();
        am.put_object(Root, "records", ObjType::List).unwrap();
        am.update_diff_cursor(); // subscriber should never receive the changes above
        Library {
            replicated: am,
            subscriber: Box::new(|_| {}),
            counter: 0,
        }
    }

    pub fn name(&self) -> String {
        let (name, _) = self.replicated.get(Root, "name").unwrap().expect("library has name");
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
        self.replicated.values(self.records_id()).map(|(_val, r_id)| {
            let (title, _) = self.replicated.get(&r_id, "title").unwrap().expect("record has title");
            let (author, _) = self.replicated.get(&r_id, "author").unwrap().expect("record has author");

            Record {
                id: RecordId(r_id),
                title: title.into_string().expect("title is a string"),
                author: author.into_string().expect("author is a string"),
            }
        })
    }

    fn add_to_set(&mut self, set_id: &ObjId, obj_type: ObjType) -> ObjId {
        let len = self.replicated.length(set_id);
        self.replicated.insert_object(set_id, len, obj_type).unwrap()
    }

    fn remove_from_set(&mut self, set_id: &ObjId, item_id: &ObjId) {
        let o_index = self.replicated.values(set_id).position(|(_, id)| id == *item_id);
        if let Some(index) = o_index {
            self.replicated.splice(set_id, index, 1, std::iter::empty::<&str>()).unwrap();
        }
    }
}

pub trait Apply {
    fn apply(&mut self, action: &Action<()>);
}

impl Apply for Library {
    fn apply(&mut self, action: &Action<()>) {
        use action::Action::*;
        match action {
            SetName(name) => {
                self.replicated.put(Root, "name", name).unwrap();
            }

            AddRecord(()) => {
                let r_id = self.add_to_set(&self.records_id(), ObjType::Map);
                // consider hydrating from Record type
                self.replicated.put(r_id, "title", self.counter.to_string()).unwrap();
                self.counter += 1;
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

        let patches = self.replicated.diff_incremental();
        for p in patches {
            (self.subscriber)(Event::from_patch(p).unwrap());
        }
    }
} 

impl Apply for Arc<Mutex<Library>> {
    fn apply(&mut self, action: &Action<()>) {
        let mut library = self.lock().unwrap();
        library.apply(action);
    }
}
