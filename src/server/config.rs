use std::error::Error;
use std::fs;

use megacommerce_shared::models::errors::{BoxedErr, ErrorType, InternalError};

use crate::{models::config::Config, server::Server};

impl Server {
  pub(crate) async fn init_service_config(&self) -> Result<(), Box<dyn Error>> {
    let ie = |err: BoxedErr, msg: &str| InternalError {
      err_type: ErrorType::ConfigError,
      temp: false,
      msg: msg.into(),
      path: "products.server.load_service_config".into(),
      err,
    };

    let yaml_string = fs::read_to_string("config.yaml")
      .map_err(|e| ie(Box::new(e), "failed to load service config file"))?;

    let parsed_config: Config = serde_yaml::from_str(&yaml_string)
      .map_err(|e| ie(Box::new(e), "failed to parse service config file"))?;

    let mut config = self.service_config.lock().await;
    *config = parsed_config;

    Ok(())
  }
}
