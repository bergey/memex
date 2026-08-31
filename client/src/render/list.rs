use super::Record;
use crate::prelude::*;
use memex_shared::library::action::Action;

use memex_query::*;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;
use std::collections::HashSet;

pub fn list_section(library: super::ReactiveLibrary, tx: Sender<Action>) -> impl IntoView {
    let mut library_add_button = tx.clone();
    let selected_id = {
        let selected = library.selected.clone();
        move || selected.get().map(|r| r.id)
    };
    section().id("list").child((
        div().class("menu").child((button()
            .on(event::click, move |_| {
                library_add_button.send(Action::AddRecord(()));
            })
            .child("Add"),)),
        table().child((
            thead().child(tr().child((th().child("Title"), th().child("Author")))),
            For(ForProps {
                // each: move || library.records.get(),
                each: move || {
                    let search = library.search.get();
                    match parse_query(&*search) {
                        Err(e) => {
                            warn!("{e}");
                            library.records.get()
                        }
                        // TODO memoize, handle invalid queries
                        Ok(query) => filter_library(query, &library.records),
                    }
                },
                key: |record| record.id.clone(),
                children: move |record| {
                    let library = library.clone();
                    tr().on(event::click, {
                        let record = record.clone();
                        move |_| {
                            library.selected.set(Some(record.clone()));
                        }
                    })
                    .class(("selected", {
                        let id = Some(record.id.clone());
                        let selected_id = selected_id.clone();
                        move || id == selected_id()
                    }))
                    .child((
                        td().child(move || record.title.get()),
                        td().child(move || record.author.get()),
                    ))
                },
            }),
        )),
    ))
}

fn filter_library(query: Query<String>, records: &RwSignal<Vec<Record>>) -> Vec<Record> {
    let mut ret = Vec::new();
    records.with(|records| {
        for r in records {
            // TODO don't rebuild this set every time the query changes
            let mut tag_set = HashSet::new();
            r.tags.with(|tags| {
                for t in tags {
                    let name = t.name.get();
                    if !name.is_empty() {
                        tag_set.insert(name);
                    }
                }
            });
            if match_tags(&query, &tag_set) {
                ret.push(r.clone());
            }
        }
    });
    ret
}
