use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
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
pub struct Error {
    error: anyhow::Error,
    status_code: StatusCode,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{}", self.error);
        (self.status_code, format!("Something went wrong")).into_response()
    }
}

impl<E> From<E> for Error
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

pub type Page = Result<Html<String>, Error>;

pub trait AsHtml {
    fn as_html(self) -> Page;
}

impl AsHtml for String {
    fn as_html(self) -> Page {
        Ok(Html::from(self))
    }
}

pub fn unwrap_or_404<T>(opt: Option<T>) -> Result<T, Error> {
    opt.ok_or(Error {
        error: anyhow::anyhow!("unwrapped None to 404"),
        status_code: StatusCode::NOT_FOUND,
    })
}

// #[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy)]
// #[sqlx(transparent)]
// pub struct RestaurantId(i32);
