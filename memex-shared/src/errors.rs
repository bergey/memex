use std::fmt::Debug;
use tracing::error;

pub trait LogResult<R> {
  fn log_error(self) -> R;
}

impl<A, E: Debug> LogResult<Option<A>> for std::result::Result<A, E> {
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

impl<E: Debug> LogResult<()> for std::result::Result<(), E> {
  fn log_error(self) -> () {
    match self {
      Ok(()) => (),
      Err(e) => error!("{:?}", e),
    }
  }
}
