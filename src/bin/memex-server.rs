use askama::Template;
use axum::{routing::get, Router};
use tokio; // bring trait in scope

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root
struct HelloTemplate<'a> {
    // the name of the struct can be anything
    name: &'a str, // the field name should match the variable name
                   // in your template
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route(
        "/",
        get(|| async {
            let hello = HelloTemplate { name: "bergey" }; // instantiate your struct
            hello.render().unwrap()
        }),
    );

    // run our app with hyper, listening globally on port 8000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
