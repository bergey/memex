use leptos::prelude::*;

pub fn body() -> impl IntoView {
    view! {
        <Search />
        <section id="list">
        <table>
            <thead>
            <tr>
                <th>Title</th>
                <th>Author</th>
            </tr>
            </thead>
            <tr>
            <td>Principles of Model Checking</td>
            </tr>
            <tr class="selected">
            <td>The Dawn of Everything</td>
            <td>David Graeber</td>
            </tr>
        </table>
        </section>
        <Details />
    }
}

fn Search() -> impl IntoView {
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

fn Details() -> impl IntoView {
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
