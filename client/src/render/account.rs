use crate::auth;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn Account() -> impl IntoView {
  // TODO track auth state some other way
  // let o_auth_token = auth::load_auth_token().await;

  // if o_auth_token.is_some() {
  //   p().child("placeholder").into_any()
    button()
      .child("sign up")
      .on(event::click, {
        let tx = super::use_worker_channel();
        move |_| {
          spawn_local(auth::signup_and_store(tx.clone()));
        }
      })
      .into_any()
}
