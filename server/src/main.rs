mod observability;
mod prelude;

use prelude::*;

use axum::extract::State;
use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
// use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    observability::init()?;

    let connection_pools = ConnectionPools {
        postgres: PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://memex:memex@localhost:5432/memex")
            .await?,
    };

    let app = Router::new()
        .route("/", get(root))
        .with_state(connection_pools);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn root(State(pools): Pools) -> Page {
    "memex placeholder".to_string().as_html()
}
