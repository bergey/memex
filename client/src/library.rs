use crate::prelude::*;

use automerge::{self, AutoCommit, ReadDoc, ObjId::Root, ObjType, transaction::Transactable};

/// for now, it only makes sense to have one Library
/// in future, it will be possible for several users to share a Library, so useful for one User to have / belong to several Libraries
pub struct Library(AutoCommit);

/// a Record may be a book, paper, movie, whatever
// TODO &str instead of owned Strings
pub struct Record {
    title: String,
}

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

    pub fn records(&self) -> impl Iterator<Item = Record> {
        let (_, records_id) = self.0
            .get(Root, "records")
            .unwrap()
            .expect("library has records");
        self.0.values(records_id).map(|(_val, r_id)| {
            let (title, _) = self.0.get(r_id, "title").unwrap().expect("record has title");

            Record {
                title: title.into_string().expect("title is a string"),
            }
        })
    }
}
