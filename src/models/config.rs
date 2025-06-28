use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub service: ServiceConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
  pub env: String,
  pub gprc_host: String,
  pub grpc_port: u16,
  pub common_service_url: String,
}
