mod details;
mod library;
mod list;

use crate::prelude::*;
use memex_shared::library::{RecordId, TagId, action::Action};

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
        <section id="searches">
            <h1>Search</h1>
            <input id="search" />
        </section>
    }
}

#[derive(Clone, Debug)]
pub struct ReactiveLibrary {
    name: RwSignal<String>,
    records: RwSignal<Vec<Record>>, // Map RecordId Record ?  (removing ID from Record)
    selected: RwSignal<Option<Record>>,
    search: RwSignal<String>,
}

// TODO decide whether leptos Stores are ready; eliminate this boilerplate
// https://book.leptos.dev/view/04b_iteration.html#option-4-stores
#[derive(Clone, Debug)]
struct Record {
    id: RecordId,
    title: RwSignal<String>,
    author: RwSignal<String>,
    url: RwSignal<String>,
    typ: RwSignal<String>,
    date: RwSignal<String>,
    date_added: RwSignal<String>,
    read_last: RwSignal<String>,
    tags: RwSignal<Vec<Tag>>,
}

#[derive(Clone, Debug)]
struct Tag {
    id: TagId,
    name: RwSignal<String>,
}
