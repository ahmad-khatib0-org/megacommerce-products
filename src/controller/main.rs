use std::{error::Error, net::SocketAddr};

use megacommerce_proto::{products_service_server::ProductsServiceServer, Config as SharedConfig};
use tonic::transport::Server as GrpcServer;
use tracing::info;

use crate::{models::errors::InternalError, utils::net::validate_url_target};

#[derive(Debug)]
pub struct Controller {
  cfg: SharedConfig,
}

#[derive(Debug)]
pub struct ControllerArgs {
  pub cfg: SharedConfig,
}

impl Controller {
  pub fn new(args: ControllerArgs) -> Controller {
    Controller { cfg: args.cfg }
  }

  pub async fn run(self) -> Result<(), Box<dyn Error>> {
    let srv = self.cfg.services.as_ref().unwrap().clone();

    let url = srv.products_service_grpc_url();
    validate_url_target(url).map_err(|e| {
      Box::new(InternalError {
        err: Box::new(e),
        temp: false,
        msg: "failed to run products service server".into(),
        path: "products.controller.run".into(),
      })
    })?;

    info!("products service server is running on: {}", url);
    GrpcServer::builder()
      .add_service(ProductsServiceServer::new(self))
      .serve((url.parse::<SocketAddr>()).unwrap())
      .await?;

    Ok(())
  }
}
