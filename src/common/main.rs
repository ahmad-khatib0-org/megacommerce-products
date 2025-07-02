use std::error::Error;
use std::sync::Arc;

use derive_more::Display;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use megacommerce_proto::{common_service_client::CommonServiceClient, Config as SharedConfig};

use crate::models::config::Config as ServiceConfig;

#[derive(Debug, Display)]
pub struct CommonArgs {
  pub service_config: ServiceConfig,
}

#[derive(Debug)]
pub struct Common {
  pub(crate) shared_config: Arc<Mutex<SharedConfig>>,
  pub(crate) service_config: ServiceConfig,
  pub(crate) client: Option<CommonServiceClient<Channel>>,
}

impl Common {
  pub async fn new(args: CommonArgs) -> Result<Common, Box<dyn Error>> {
    let mut common = Common {
      shared_config: Arc::new(Mutex::new(SharedConfig::default())),
      service_config: args.service_config,
      client: None,
    };

    match common.init_common_client().await {
      Ok(cli) => common.client = Some(cli),
      Err(e) => return Err(e),
    }

    Ok(common)
  }

  /// Close the client connection by dropping it
  pub fn close(&mut self) {
    self.client = None;
    // When `client` is dropped, the underlying Channel drops and closes the connection.
  }

  /// Reconnect: close old client if any, then create new one
  pub async fn reconnect(&mut self) -> Result<(), Box<dyn Error>> {
    self.close(); // drop old client if present
    let client = self.init_common_client().await?;
    self.client = Some(client);
    Ok(())
  }

  /// Accessor for client with error if not connected
  pub fn client(&mut self) -> Result<&mut CommonServiceClient<Channel>, Box<dyn Error>> {
    self
      .client
      .as_mut()
      .ok_or_else(|| "client not connected".into())
  }
}
