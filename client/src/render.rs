mod details;
mod library;
mod list;

use crate::prelude::*;
use memex_shared::library::{RecordId, action::Action};

use leptos::html::*;
use leptos::prelude::*;

pub fn body(reactive: ReactiveLibrary, tx: Sender<Action>) -> impl IntoView {
    (
        search(),
        list::list_section(reactive.clone(), tx.clone()),
        details::details(tx, reactive.selected),
    )
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
