mod details;
mod library;
mod list;

use crate::prelude::*;
use memex_shared::library::{RecordId, TagId, action::Action};

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use std::collections::HashSet;

// how to trigger WS connect after auth?  New channel?
pub fn router(reactive: ReactiveLibrary, tx: Sender<Action>) -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Not found.">
            <Route path=path!("/") view= move || {
                let reactive = reactive.clone();
                let tx = tx.clone();
                body(reactive, tx) // list of libraries & saved searches, none selected
            } />
            // deep links to selected library / card
            <Route path=path!("/lib/:library_id") view=|| {}/>
            <Route path=path!("/lib/:library_id/card/:r_id") view=|| {}/>
            <Route path=path!("/login") view=|| { } /* redirect to signup if no passkeys match */ />
            <Route path=path!("/signup") view=|| { /* show links to /signup/new & /login/ask */ }/>
            <Route path=path!("/signup/new") view=|| { /* passkey signup flow */ }/>
            <Route path=path!("/login/ask") view=|| { /* display slug URL & QR code from server  */ }/>
            <Route path=path!("/login/:slug") view=|| { /* prompt user to auth the other browser */ }/>
            </Routes>
        </Router>
    }
    // Router(RouterProps {
    //     base: None,
    //     set_is_routing: None,
    //     children: move || Route(RouteProps {}),
    // })
}

// How should this function get the ReactiveLibrary & Sender?
// neither can be passed in the URL
pub fn body(reactive: ReactiveLibrary, tx: Sender<Action>) -> impl IntoView {
    (
        search(reactive.clone()),
        list::list_section(reactive.clone(), tx.clone()),
        details::details(tx, reactive.selected),
    )
}

fn search(library: ReactiveLibrary) -> impl IntoView {
    section().id("searches").child((
        button()
            .child("Sync") // TODO icon
            .on(event::click, move |_| {
                // connect, with auth token
                // refresh token someday
                // with no auth token, or token expired, try login
                // if login fails, offer signup / link to existing
            }),
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
