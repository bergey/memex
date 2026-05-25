use leptos::html::*;
use leptos::prelude::*;

pub fn body() -> impl IntoView {
    (search(), list_section(), details())
}

fn list_section() -> impl IntoView {
    section().id("list").child(table().child((
        thead().child(tr().child((th().child("Title"),))),
        tr().child((td().child("Principles of Model Checking"),)),
        tr().child((td().child("The Dawn of Everything"),)),
    )))
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

fn details() -> impl IntoView {
    view! {
        <section id="details">
            <h1>Details</h1>
            <ul>
                <li><label>Title <input id="title" type="text" value="The Dawn of Everything" /></label></li>
                <li><label>Author <input id="author" type="text" value="David Graeber" /></label></li>
                <li><label>Date <input id="date" type="text" /></label></li>
                <li><label>Publisher <input id="publisher" type="text" /></label></li>
                <li><label>URL <input id="url" type="text" /></label></li>
            </ul>
        </section>
    }
}
