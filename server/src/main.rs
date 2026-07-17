mod observability;
mod prelude;

use prelude::*;

use axum::{
    Router,
    extract::{
        State,
        ws::{self, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::get,
};
use sqlx::postgres::PgPoolOptions;
// use tokio::try_join;

async fn ws_upgrade(State(_database): Pools, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| echo(ws))
}

async fn echo(mut socket: WebSocket) {
    loop {
        let o_msg = socket.recv().await;
        match o_msg {
            Some(Ok(ws_msg)) => {
                debug!("{:?}", ws_msg);
                if let Err(e) = socket.send(ws_msg).await {
                    error!("{e}");
                    break;
                }
            }
            _ => break,
        }
    }
}

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
        .route("/ws", get(ws_upgrade))
        .with_state(connection_pools);
    // TODO serve client code

    let addr = "0.0.0.0:3036"; // TODO env var
    info!("memex listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn root(State(_pools): Pools) -> Page {
    "memex placeholder".to_string().as_html()
}
