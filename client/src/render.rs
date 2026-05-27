use crate::library::{action::Action, Apply, Library, LibraryRef, RecordId};
use crate::prelude::*;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;

pub fn body(library_ref: LibraryRef) -> impl IntoView {
    let reactive = {
        let mut library = library_ref.lock().unwrap();
        let reactive = ReactiveLibrary::from_replicated(&*library);
        library.subscriber = {
            let mut reactive = reactive.clone();
            Box::new(move |ev| reactive.apply(ev))
        };
        reactive
    };

    (
        search(),
        list_section(reactive.clone(), library_ref.clone()),
        details(library_ref, reactive.selected),
    )
}

fn list_section(library: ReactiveLibrary, library_ref: LibraryRef) -> impl IntoView {
    let mut library_add_button = library_ref.clone();
    section().id("list").child((
        button()
            .on(event::click, move |_| {
                library_add_button.apply(&Action::AddRecord(()));
            })
            .child("Add Record"),
        table().child((
            thead().child(tr().child((th().child("Title"),))),
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
                    .child((td().child(move || record.title.get()),))
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

fn details(library_ref: LibraryRef, selected: RwSignal<Option<Record>>) -> impl IntoView {
    // update CRDT only on blur.  Accept lost edits if remove change comes through before blur
    // someday if I build a history UI that suppors AM get_all / manual conflict resolution, consider flushing dirty fields
    move || {
        let mut library_ref = library_ref.clone();
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
                            )) // value(selected.title.get()))),
                            .on(event::change, move |_| {
                                debug!("left title");
                                library_ref.apply(&Action::SetTitle(
                                    selected.id.clone(),
                                    selected.title.get(),
                                ));
                            }),
                    ),
                    // <li><label>Author <input id="author" type="text" value="David Graeber" /></label></li>
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
struct ReactiveLibrary {
    name: RwSignal<String>,
    records: RwSignal<Vec<Record>>, // Map RecordId Record ?  (removing ID from Record)
    selected: RwSignal<Option<Record>>,
}

#[derive(Clone, Debug)]
struct Record {
    id: RecordId,
    title: RwSignal<String>,
}

impl ReactiveLibrary {
    fn from_replicated(library: &Library) -> ReactiveLibrary {
        ReactiveLibrary {
            name: RwSignal::new(library.name()),
            selected: RwSignal::new(None),
            records: RwSignal::new(
                library
                    .records()
                    .map(|record| Record {
                        // TODO extra clone here, because types don't reflect unique Library::Record
                        // make Record &str instead
                        id: record.id.clone(),
                        title: RwSignal::new(record.title.clone()),
                    })
                    .collect(),
            ),
        }
    }

    fn apply(&mut self, action: crate::library::action::Event) {
        debug!("entering ReactiveLibrary::apply: {:?}", action);
        use crate::library::action::Action::*;
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

            DeleteRecord(index) => self.records.update(|rs| {
                rs.remove(index);
            }),
        }
    }
}
