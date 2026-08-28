use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
pub use tracing::*;

pub type Pools = State<ConnectionPools>;

#[derive(Clone)]
pub struct ConnectionPools {
    pub postgres: PgPool,
}

// TODO trait to make State wrapper less intrusive?

// https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs
pub struct HttpError {
    pub error: anyhow::Error,
    pub status_code: StatusCode,
}

pub type HttpResult<T> = std::result::Result<T, HttpError>;

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        error!("{}", self.error);
        (self.status_code, format!("Something went wrong")).into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            error: err.into(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[allow(dead_code)]
pub trait WithStatus<T> {
    fn with_status(self, status: StatusCode) -> HttpResult<T>;
}

impl<E: Into<anyhow::Error>, T> WithStatus<T> for std::result::Result<T, E> {
    fn with_status(self, status_code: StatusCode) -> HttpResult<T> {
        self.map_err(|e| HttpError {
            error: e.into(),
            status_code,
        })
    }
}
