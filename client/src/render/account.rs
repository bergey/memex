use crate::auth;

use leptos::html::*;
use leptos::prelude::*;
use leptos::tachys::html::event;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn Account() -> impl IntoView {
  let o_auth_token = auth::load_auth_token();

  if o_auth_token.is_some() {
    p().child("placeholder").into_any()
  } else {
    button()
      .child("sign up")
      .on(event::click, move |_| {
        spawn_local(auth::signup_and_store());
      })
      .into_any()
  }
}
