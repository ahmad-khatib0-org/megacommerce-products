use std::fs;

use crate::models::{config::Config, errors::InternalError};
use crate::server::server::Server;

impl Server {
  pub async fn init_service_config(&self) {
    let yaml_string = match fs::read_to_string("config.yaml") {
      Ok(s) => s,
      Err(err) => {
        let _ = self
          .errors
          .send(InternalError {
            temp: false,
            msg: "failed to load service config file".into(),
            path: "products.server.load_service_config".into(),
            err: Box::new(err),
          })
          .await;
        return;
      }
    };

    let parsed_config: Config = match serde_yaml::from_str(&yaml_string) {
      Ok(cfg) => cfg,
      Err(e) => {
        let _ = self
          .errors
          .send(InternalError {
            temp: false,
            msg: "failed to parse config data".into(),
            path: "products.server.load_service_config".into(),
            err: Box::new(e),
          })
          .await;
        return;
      }
    };

    let mut config = self.config.lock().await;
    // config.
    *config = parsed_config;
  }
}
