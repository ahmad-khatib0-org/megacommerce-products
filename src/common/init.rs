use std::error::Error;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::Channel;

use megacommerce_proto::{common_service_client::CommonServiceClient, Config as SharedConfig};

use crate::models::config::Config as ServiceConfig;

#[derive(Debug)]
pub struct CommonArgs {
  service_config: ServiceConfig,
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
}
