use std::error::Error;
use std::fs;

use crate::{
  models::{config::Config, errors::InternalError},
  server::Server,
};

impl Server {
  pub(crate) async fn init_service_config(&self) -> Result<(), Box<dyn Error>> {
    let yaml_string = fs::read_to_string("config.yaml").map_err(|e| InternalError {
      temp: false,
      msg: "failed to load service config file".into(),
      path: "products.server.load_service_config".into(),
      err: Box::new(e),
    })?;

    let parsed_config: Config = serde_yaml::from_str(&yaml_string).map_err(|e| InternalError {
      temp: false,
      msg: "failed to parse config data".into(),
      path: "products.server.load_service_config".into(),
      err: Box::new(e),
    })?;

    let mut config = self.service_config.lock().await;
    *config = parsed_config;

    Ok(())
  }
}
