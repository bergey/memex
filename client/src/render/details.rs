use super::Record;
use crate::prelude::*;
use memex_shared::library::{
    RecordId,
    action::{Action, RecordField},
    date::Date,
};

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;

pub fn details(tx: Sender<Action>, selected: RwSignal<Option<Record>>) -> impl IntoView {
    // update CRDT only on blur.  Accept lost edits if remote change comes through before blur
    // someday if I build a history UI that supports AM get_all / manual conflict resolution, consider flushing dirty fields
    move || {
        if let Some(selected) = selected.read().clone() {
            let text_field = {
                let tx = tx.clone();
                let id = selected.id.clone();
                move |name: &'static str,
                      signal: RwSignal<String>,
                      action: &'static dyn Fn(String) -> RecordField| {
                    field(
                        tx.clone(),
                        id.clone(),
                        name,
                        signal,
                        action,
                        |s| s.clone(),
                        |s| s.to_owned(),
                    )
                }
            };

            let date_field = {
                let tx = tx.clone();
                let id = selected.id.clone();
                move |name: &'static str,
                      signal: RwSignal<Date>,
                      action: &'static dyn Fn(Date) -> RecordField| {
                    field(
                        tx.clone(),
                        id.clone(),
                        name,
                        signal,
                        action,
                        Date::to_string,
                        Date::from_str,
                    )
                }
            };

            section()
                .id("details")
                .child((ul().child((
                    text_field("Title", selected.title, &RecordField::Title),
                    text_field("Author", selected.author, &RecordField::Author),
                    text_field("Type", selected.typ, &RecordField::Type),
                    text_field("URL", selected.url, &RecordField::Url),
                    date_field("Date", selected.date, &RecordField::Date),
                    date_field("Date Added", selected.date_added, &RecordField::DateAdded),
                    date_field("Read Last", selected.read_last, &RecordField::ReadLast),
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

fn field<T, F, G>(
    tx: Sender<Action>,
    id: RecordId,
    name: &'static str,
    field: RwSignal<T>,
    action: &'static dyn Fn(T) -> RecordField,
    to_string: F,
    from_string: G,
) -> impl IntoView
where
    F: Fn(&T) -> String,
    G: Fn(&str) -> T + 'static,
    T: Sync + Send + Clone + 'static,
{
    let input_element: NodeRef<Input> = NodeRef::new();

    li().child(
        label()
            .child((
                name,
                input()
                    .id(name) // TODO downcase_snake
                    .value(to_string(&field.get()))
                    .node_ref(input_element),
            ))
            .on(event::change, {
                let mut tx = tx.clone();
                let id = id.clone();
                move |_| {
                    let val = from_string(input_element.get().unwrap().value().as_ref());
                    tx.send(Action::SetRecord(id.clone(), action(val)));
                }
            }),
    )
}
