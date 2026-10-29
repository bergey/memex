use crate::auth;

use leptos::html::*;
use leptos::prelude::*;

#[component]
fn Account() -> impl IntoView {
  let o_auth_token = auth::load_auth_token();

  if o_auth_token.is_some() {
    view! {<p>placeholder</p> }.into_any()
  } else {
    view! {<button>sign up</button> }.into_any()
  }
}
