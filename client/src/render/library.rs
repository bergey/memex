use super::{ReactiveLibrary, Record, Tag};
use crate::prelude::*;
use memex_shared::library::{self, Library, RecordId, action::Event};

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
                        date: RwSignal::new(record.date.to_string()),
                        date_added: RwSignal::new(record.date_added.to_string()),
                        read_last: RwSignal::new(record.read_last.to_string()),
                        tags: RwSignal::new(
                            record
                                .tags
                                .into_iter()
                                .map(|library::Tag { id, name }| Tag {
                                    id,
                                    name: RwSignal::new(name),
                                })
                                .collect(),
                        ),
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
                        date: RwSignal::new(Default::default()),
                        date_added: RwSignal::new(Default::default()),
                        read_last: RwSignal::new(Default::default()),
                        tags: RwSignal::new(Vec::new()),
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
                            Date(v) => r.date.set(v.to_string()),
                            DateAdded(v) => r.date_added.set(v.to_string()),
                            ReadLast(v) => r.read_last.set(v.to_string()),
                        }
                        break;
                    }
                }
            }),

            DeleteRecord(index) => self.records.update(|rs| {
                rs.remove(index);
            }),

            AddTag(r_id, t_id) => {
                self.update_record(&r_id, |r| {
                    r.tags.update(|tags| {
                        tags.push(Tag {
                            id: t_id.clone(),
                            name: RwSignal::new("".to_string()),
                        });
                    });
                });
            }

            DeleteTag(r_id, ix) => {
                self.update_record(&r_id, |r| {
                    r.tags.update(|tags| {
                        tags.remove(ix);
                    });
                });
            }

            SetTag(r_id, t_id, s) => {
                self.update_record(&r_id, |r| {
                    r.tags.update(|tags| {
                        let o_ix = tags.iter().position(|t| t.id == t_id);
                        if let Some(ix) = o_ix {
                            tags[ix].name.set(s.clone());
                        }
                    });
                });
            }
        }
    }

    fn update_record<F>(&mut self, r_id: &RecordId, block: F) -> bool
    where
        F: Fn(&mut Record),
    {
        let mut ret = false;
        self.records.update(|rs| {
            for r in rs {
                if r.id == *r_id {
                    block(r);
                    ret = true;
                }
            }
        });
        ret
    }
}
