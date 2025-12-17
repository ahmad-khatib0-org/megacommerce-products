use std::error::Error;
use std::{env, fs};

use megacommerce_shared::models::errors::{BoxedErr, ErrorType, InternalError};

use crate::{models::config::Config, server::Server};

impl Server {
  pub(crate) async fn init_service_config(&self) -> Result<(), Box<dyn Error>> {
    let mut env_mode = env::var("ENV").unwrap_or("local".to_string());
    if !["local", "dev", "production"].contains(&env_mode.as_str()) {
      env_mode = "local".to_string();
    }

    let ie = |err: BoxedErr, msg: &str| InternalError {
      err_type: ErrorType::ConfigError,
      temp: false,
      msg: msg.into(),
      path: "products.server.load_service_config".into(),
      err,
    };

    let yaml_string = fs::read_to_string(format!("config.{}.yaml", env_mode))
      .map_err(|e| ie(Box::new(e), "failed to load service config file"))?;

    let parsed_config: Config = serde_yaml::from_str(&yaml_string)
      .map_err(|e| ie(Box::new(e), "failed to parse service config file"))?;

    let mut config = self.service_config.lock().await;
    *config = parsed_config;

    Ok(())
  }
}
