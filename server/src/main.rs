mod database;
mod metrics;
mod observability;
mod passkeys;
mod prelude;
mod websocket;

use prelude::*;

use axum::routing::{get, post};
use sqlx::postgres::PgPoolOptions;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    observability::init()?;

    let connection_pools = ConnectionPools {
        postgres: PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://memex:memex@localhost:5432/memex")
            .await?,
    };

    let app = axum::Router::new()
        .route("/ws", get(websocket::ws_upgrade))
        .route("/signup/start", get(passkeys::signup_start))
        .route("/signup/finish", post(passkeys::signup_finish))
        .route("/login/{user_id}/start", get(passkeys::login_start))
        .route("/login/finish", post(passkeys::login_finish))
        .with_state(connection_pools);
    // TODO serve client code

    let addr = "0.0.0.0:3036"; // TODO env var
    info!("memex listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
