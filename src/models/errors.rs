use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct InternalError {
  pub temp: bool,
  pub err: Box<dyn Error + Send + Sync>,
  pub msg: String,
  pub path: String,
}

impl fmt::Display for InternalError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} (path: {}, err: {})", self.msg, self.path, self.err)
  }
}

impl Error for InternalError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(&*self.err)
  }
}
