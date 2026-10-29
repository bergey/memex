mod account;
mod details;
mod library;
mod list;

use crate::prelude::*;
use memex_shared::library::{RecordId, TagId, action::Action};

use leptos::html::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use std::collections::HashSet;

#[component]
pub fn Body(reactive: ReactiveLibrary, tx: ToWorker<Action>) -> impl IntoView {
  provide_context(reactive.clone());
  provide_context(tx.clone());
  view! {
    <Router>
      <Routes fallback=|| "Not found" >
      <Route path=path!("/") view=move || {
        library_page(reactive.clone(), tx.clone())
      } />
      </Routes>
    </Router>
  }
}

fn library_page(reactive: ReactiveLibrary, tx: ToWorker<Action>) -> impl IntoView {
  (
    search(reactive.clone()),
    list::list_section(reactive.clone(), tx.clone()),
    details::details(tx, reactive.selected),
  )
}

fn search(library: ReactiveLibrary) -> impl IntoView {
  section().id("searches").child((
    h1().child("Search"),
    // TODO inidate when search term is invalid / results are out of date
    input().bind(leptos::attr::Value, library.search),
  ))
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
#[derive(Clone, Debug, PartialEq)]
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
  tag_set: Memo<HashSet<String>>,
}

#[derive(Clone, Debug)]
struct Tag {
  id: TagId,
  name: RwSignal<String>,
}

fn use_library() -> Option<ReactiveLibrary> {
  use_context()
}

fn use_worker_channel() -> Option<ToWorker<Action>> {
  use_context()
}
