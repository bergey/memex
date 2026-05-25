mod action;

use action::*;
use crate::prelude::*;

use automerge::{self, AutoCommit, ReadDoc, ObjId, ObjId::Root, ObjType, transaction::Transactable};

/// for now, it only makes sense to have one Library
/// in future, it will be possible for several users to share a Library, so useful for one User to have / belong to several Libraries
pub struct Library(AutoCommit);

/// a Record may be a book, paper, movie, whatever
// TODO &str instead of owned Strings
pub struct Record {
    id: RecordId,
    id_to_delete: usize,
    title: String,
}
pub struct RecordId(ObjId);

impl Library {
    pub fn new() -> Self {
        let mut am = AutoCommit::new();
        am.put(Root, "name", "My Library").unwrap();
        am.put_object(Root, "records", ObjType::List).unwrap();
        Library(am)
    }

    pub fn name(&self) -> String {
        let (name, _) = self.0.get(Root, "name").unwrap().expect("library has name");
        name.into_string().expect("name is a string")
    }

    fn records_id(&self) -> ObjId {
        self.0
            .get(Root, "records")
            .unwrap()
            .expect("library has records")
            .1
    }

    pub fn records(&self) -> impl Iterator<Item = Record> {
        self.0.values(self.records_id()).enumerate().map(|(ix, (_val, r_id))| {
            let (title, _) = self.0.get(&r_id, "title").unwrap().expect("record has title");

            Record {
                id: RecordId(r_id),
                id_to_delete: ix,
                title: title.into_string().expect("title is a string"),
            }
        })
    }

    pub fn apply(&mut self, action: &Action) {
        use action::Action::*;
        match action {
            SetName(name) => {
                self.0.put(Root, "name", name).unwrap();
            }

            AddRecord() => {
                let r_id = self.add_to_set(&self.records_id(), ObjType::Map);
                // let r_id = self.0.insert_object(self.records_id(), 0, ObjType::Map).unwrap();
                // consider hydrating from Record type
                self.0.put(r_id, "title", "").unwrap();
            }

            SetTitle(RecordId(r_id), title) => {
                self.0.put(r_id, "title", title).unwrap();
            }

            DeleteRecord(index) => {
                self.remove_from_set(&self.records_id(), *index);
            }
        }
    }

    fn add_to_set(&mut self, set_id: &ObjId, obj_type: ObjType) -> ObjId {
        self.0.insert_object(set_id, 0, obj_type).unwrap()
    }

    // delete by position seems racy.
    // TODO ask automerge library devs if there's a better way
    fn remove_from_set(&mut self, set_id: &ObjId, index: usize) {
        self.0.splice(set_id, index, 1, std::iter::empty::<&str>());
    }
}
