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
                        url: RwSignal::new(record.url),
                        typ: RwSignal::new(record.typ),
                        date: RwSignal::new(record.date),
                        date_added: RwSignal::new(record.date_added),
                        read_last: RwSignal::new(record.read_last),
                    })
                    .collect(),
            ),
        }
    }

    pub fn apply(&mut self, action: Event) {
        debug!(?action, "entering ReactiveLibrary::apply");
        use memex_shared::library::action::{Action::*, RecordField::*};
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
                        url: RwSignal::new("".to_string()),
                        typ: RwSignal::new("".to_string()),
                        date: RwSignal::new(None),
                        date_added: RwSignal::new(None),
                        read_last: RwSignal::new(None),
                    },
                );
            }),

            SetRecord(r_id, field) => self.records.update(|rs| {
                for r in rs {
                    if r.id == r_id {
                        match field {
                            Title(v) => r.title.set(v),
                            Author(v) => r.author.set(v),
                            Url(v) => r.url.set(v),
                            Type(v) => r.typ.set(v),
                            Date(v) => r.date.set(Some(v)),
                            DateAdded(v) => r.date_added.set(Some(v)),
                            ReadLast(v) => r.read_last.set(Some(v)),
                        }
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
