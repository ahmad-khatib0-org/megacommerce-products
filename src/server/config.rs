use std::fs;

use crate::models::{config::Config, errors::InternalError};
use crate::server::server::Server;

impl Server {
  pub fn init_service_config(&self) {
    let yaml_string = match fs::read_to_string("config.yaml") {
      Ok(s) => s,
      Err(e) => {
        let _ = self.errors.send(InternalError {
          temp: false,
          msg: "failed to load service config file".into(),
          path: "products.server.load_service_config".into(),
          err: Box::new(e),
        });
        return;
      }
    };

    let parsed_config: Config = match serde_yaml::from_str(&yaml_string) {
      Ok(cfg) => cfg,
      Err(e) => {
        let _ = self.errors.send(InternalError {
          temp: false,
          msg: "failed to parse config data".into(),
          path: "products.server.load_service_config".into(),
          err: Box::new(e),
        });
        return;
      }
    };

    *self.config.borrow_mut() = parsed_config;
  }
}
