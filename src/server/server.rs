use std::cell::RefCell;
use std::{rc::Rc, sync::mpsc};

use crate::models::config::Config;

use crate::models::errors::InternalError;

pub struct Server {
  pub errors: mpsc::Sender<InternalError>,
  pub config: Rc<RefCell<Config>>,
}
