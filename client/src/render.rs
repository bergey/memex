use memex_shared::library::{
    Library, RecordId,
    action::{Action, Event},
};
use crate::prelude::*;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;

pub fn body(
    reactive: ReactiveLibrary,
    tx: Sender<Action>,
) -> impl IntoView {
    (
        search(),
        list_section(reactive.clone(), tx.clone()),
        details(tx, reactive.selected),
    )
}

fn list_section(library: ReactiveLibrary, tx: Sender<Action>) -> impl IntoView {
    let mut library_add_button = tx.clone();
    section().id("list").child((
        button()
            .on(event::click, move |_| {
                library_add_button.send(Action::AddRecord(()));
            })
            .child("Add Record"),
        table().child((
            thead().child(tr().child((th().child("Title"), th().child("Author")))),
            For(ForProps {
                each: move || library.records.get(),
                key: |record| record.id.clone(),
                children: move |record| {
                    let library = library.clone();
                    tr().on(event::click, {
                        let record = record.clone();
                        move |_| {
                            library.selected.set(Some(record.clone()));
                        }
                    })
                    .child((
                        td().child(move || record.title.get()),
                        td().child(move || record.author.get()),
                    ))
                },
            }),
        )),
    ))
}

fn search() -> impl IntoView {
    view! {
        <section id="search">
            <h1>Search</h1>
            <menu>
                <li>Craft</li>
                <li>Science</li>
                <li>Society</li>
                <li>Software</li>
            </menu>
        </section>
    }
}

fn details(tx: Sender<Action>, selected: RwSignal<Option<Record>>) -> impl IntoView {
    // update CRDT only on blur.  Accept lost edits if remove change comes through before blur
    // someday if I build a history UI that suppors AM get_all / manual conflict resolution, consider flushing dirty fields
    move || {
        if let Some(selected) = selected.read().clone() {
            section()
                .id("details")
                .child((ul().child((
                    li().child(
                        label()
                            .child((
                                "Title",
                                input()
                                    .id("title")
                                    .bind(leptos::attr::Value, selected.title),
                            ))
                            .on(event::change, {
                                let mut tx = tx.clone();
                                let selected = selected.clone();
                                move |_| {
                                    tx.send(Action::SetTitle(
                                        selected.id.clone(),
                                        selected.title.get(),
                                    ));
                                }
                            }),
                    ),
                    li().child(
                        label()
                            .child((
                                "Author",
                                input()
                                    .id("author")
                                    .bind(leptos::attr::Value, selected.author),
                            )) // value(selected.author.get()))),
                            .on(event::change, {
                                let mut tx = tx.clone();
                                move |_| {
                                    tx.send(Action::SetAuthor(
                                        selected.id.clone(),
                                        selected.author.get(),
                                    ));
                                }
                            }),
                    ),
                    // <li><label>Date <input id="date" type="text" /></label></li>
                    // <li><label>Publisher <input id="publisher" type="text" /></label></li>
                    // <li><label>URL <input id="url" type="text" /></label></li>
                )),))
                .into_any()
        } else {
            section()
                .id("details")
                .child((h1().child("Details"),))
                .into_any()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReactiveLibrary {
    name: RwSignal<String>,
    records: RwSignal<Vec<Record>>, // Map RecordId Record ?  (removing ID from Record)
    selected: RwSignal<Option<Record>>,
}

#[derive(Clone, Debug)]
struct Record {
    id: RecordId,
    title: RwSignal<String>,
    author: RwSignal<String>,
}

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
        debug!("entering ReactiveLibrary::apply: {:?}", action);
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
