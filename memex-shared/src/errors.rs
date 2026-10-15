use std::fmt::Debug;
use tracing::error;

pub trait LogResult<A> {
  fn log_error(self) -> Option<A>;
}

impl<A, E: Debug> LogResult<A> for std::result::Result<A, E> {
  fn log_error(self) -> Option<A> {
    match self {
      Ok(a) => Some(a),
      Err(e) => {
        error!("{:?}", e);
        None
      }
    }
  }
}
