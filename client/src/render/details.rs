use super::Record;
use memex_shared::library::action::Action;
use crate::prelude::*;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;

pub fn details(tx: Sender<Action>, selected: RwSignal<Option<Record>>) -> impl IntoView {
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
