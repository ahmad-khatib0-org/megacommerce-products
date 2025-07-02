use std::{error::Error, time::Duration};

use megacommerce_proto::{common_service_client::CommonServiceClient, PingRequest};
use tokio::time::timeout;
use tonic::{transport::Channel, Request};

use super::main::Common;
use crate::{models::errors::InternalError, utils::net::validate_url_target};

impl Common {
  pub(super) async fn init_common_client(
    &mut self,
  ) -> Result<CommonServiceClient<Channel>, Box<dyn Error>> {
    let mk_err = |msg: &str, err: Box<dyn Error + Send + Sync>| {
      Box::new(InternalError {
        temp: false,
        msg: msg.into(),
        path: "products.common.init_common_client".into(),
        err,
      }) as Box<dyn Error>
    };

    let url = self.service_config.service.common_service_grpc_url.clone();
    if let Err(e) = validate_url_target(&url) {
      return Err(mk_err("failed to validate common client URL", Box::new(e)));
    }

    let mut client = CommonServiceClient::connect(url)
      .await
      .map_err(|e| mk_err("failed to connect to common client", Box::new(e)))?;

    let request = Request::new(PingRequest {});
    let respones = timeout(Duration::from_secs(5), client.ping(request)).await;
    match respones {
      Ok(Ok(_)) => {}
      Ok(Err(e)) => {
        return Err(mk_err(
          "failed to ping the common client service",
          Box::new(e),
        ))
      }
      Err(e) => {
        return Err(mk_err(
          "the ping to common client service timedout",
          Box::new(e),
        ))
      }
    };

    Ok(client)
  }
}
