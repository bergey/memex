use super::{ReactiveLibrary, Record};
use crate::prelude::*;
use memex_shared::library::{Library, action::Event};

use leptos::prelude::*;


impl ReactiveLibrary {
    pub fn from_replicated(library: &Library) -> ReactiveLibrary {
        ReactiveLibrary {
            name: RwSignal::new(library.name()),
            selected: RwSignal::new(None),
            records: RwSignal::new(
                library
                    .records()
                    .map(|record| Record {
                        id: record.id,
                        title: RwSignal::new(record.title),
                        author: RwSignal::new(record.author),
                    })
                    .collect(),
            ),
        }
    }

    pub fn apply(&mut self, action: Event) {
        debug!(?action, "entering ReactiveLibrary::apply");
        use memex_shared::library::action::Action::*;
        match action {
            SetName(name) => {
                self.name.set(name);
            }

            AddRecord((ix, r_id)) => self.records.update(|rs| {
                rs.insert(
                    ix,
                    Record {
                        id: r_id,
                        title: RwSignal::new("".to_string()),
                        author: RwSignal::new("".to_string()),
                    },
                );
            }),

            SetTitle(r_id, title) => self.records.update(|rs| {
                for r in rs {
                    if r.id == r_id {
                        r.title.set(title);
                        break;
                    }
                }
            }),

            SetAuthor(r_id, author) => self.records.update(|rs| {
                for r in rs {
                    if r.id == r_id {
                        r.author.set(author);
                        break;
                    }
                }
            }),

            DeleteRecord(index) => self.records.update(|rs| {
                rs.remove(index);
            }),
        }
    }
}
