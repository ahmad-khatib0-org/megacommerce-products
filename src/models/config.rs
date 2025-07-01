use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
  pub service: ServiceConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServiceConfig {
  pub env: String,
  pub gprc_host: String,
  pub grpc_port: u16,
  pub common_service_url: String,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      service: ServiceConfig {
        env: "".to_string(),
        gprc_host: "".to_string(),
        grpc_port: 0,
        common_service_url: "".to_string(),
      },
    }
  }
}
