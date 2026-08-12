use crate::prelude::*;
use memex_shared::library::action::Action;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;

pub fn list_section(library: super::ReactiveLibrary, tx: Sender<Action>) -> impl IntoView {
    let mut library_add_button = tx.clone();
    let selected_id = {
        let selected = library.selected.clone();
        move || selected.get().map(|r| r.id)
    };
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
