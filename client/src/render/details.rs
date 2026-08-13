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
                    field(tx.clone(), id.clone(), name, signal, action, &identity)
                }
            };

            let date_field = {
                let tx = tx.clone();
                let id = selected.id.clone();
                move |name: &'static str,
                      signal: RwSignal<String>,
                      action: &'static dyn Fn(Date) -> RecordField| {
                    field(
                        tx.clone(),
                        id.clone(),
                        name,
                        signal,
                        action,
                        &Date::from_str,
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
                    div().id("tags").child((
                        button()
                            .on(event::click, {
                                let mut tx = tx.clone();
                                let id = selected.id.clone();
                                move |_| tx.send(Action::AddTag(id.clone(), ()))
                            })
                            .child("Add Tag"),
                        ul().child(For(ForProps {
                            each: move || selected.tags.get(),
                            key: |tag| tag.id.clone(),
                            children: {
                                let tx = tx.clone();
                                move |tag| {
                                    li().child((
                                        input().bind(leptos::attr::Value, tag.name).on(
                                            event::change,
                                            {
                                                let mut tx = tx.clone();
                                                let r_id = selected.id.clone();
                                                let t_id = tag.id.clone();
                                                move |_| {
                                                    tx.send(Action::SetTag(
                                                        r_id.clone(),
                                                        t_id.clone(),
                                                        tag.name.get(),
                                                    ))
                                                }
                                            },
                                        ),
                                        button().child("-").on(event::click, {
                                                let mut tx = tx.clone();
                                                let r_id = selected.id.clone();
                                                let t_id = tag.id.clone();
                                                move |_| {
                                                    tx.send(Action::DeleteTag(
                                                        r_id.clone(),
                                                        t_id.clone(),
                                                    ))
                                                }
                                            }
                                        ),
                                    ))
                                }
                            },
                        })),
                    )),
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

fn field<T>(
    tx: Sender<Action>,
    id: RecordId,
    name: &'static str,
    field: RwSignal<String>,
    // keep these separate rather than define a 'static fn for each RecordField constructor
    action: &'static dyn Fn(T) -> RecordField,
    from_string: &'static dyn Fn(String) -> T,
) -> impl IntoView {
    li().child(
        label()
            .child((
                name,
                input()
                    .id(name) // TODO downcase_snake
                    .bind(leptos::attr::Value, field),
            ))
            .on(event::change, {
                let mut tx = tx.clone();
                let id = id.clone();
                move |_| {
                    tx.send(Action::SetRecord(
                        id.clone(),
                        action(from_string(field.get())),
                    ));
                }
            }),
    )
}

fn identity<T>(t: T) -> T {
    t
}
