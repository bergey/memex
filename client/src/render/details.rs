use super::Record;
use memex_shared::library::action::{Action, RecordField};
use crate::prelude::*;

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
                move |name: &'static str, field: RwSignal<String>, action: &'static dyn Fn(String) -> RecordField| {
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
                                let field = field.clone();
                                move |_| {
                                    tx.send(Action::SetRecord(
                                        id.clone(),
                                        action(field.get()),
                                    ));
                                }
                            }),
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

                    // <li><label>Date <input id="date" type="text" /></label></li>
                    // <li><label>Publisher <input id="publisher" type="text" /></label></li>
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

// fn text_field(label: &str, id: &RecordId, )
