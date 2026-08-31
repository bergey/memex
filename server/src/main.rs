mod database;
mod metrics;
mod observability;
mod passkeys;
mod prelude;
mod websocket;

use prelude::*;

use axum::routing::{get, post};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    observability::init()?;

    let connection_pools = ConnectionPools {
        postgres: PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("DATABASE_URL").unwrap_or("postgres://memex:memex@localhost:5432/memex".to_string()))
            .await?,
    };
    sqlx::migrate!().run(&connection_pools.postgres).await?;

    let app = axum::Router::new()
        .route("/ws", get(websocket::ws_upgrade))
        .route("/signup/start", get(passkeys::signup_start))
        .route("/signup/finish", post(passkeys::signup_finish))
        .route("/login/{user_id}/start", get(passkeys::login_start))
        .route("/login/finish", post(passkeys::login_finish))
        .with_state(connection_pools);

    // TODO require env var in release build
    let addr = env::var("LISTEN").unwrap_or("0.0.0.0:3036".to_string());
    info!("memex listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
