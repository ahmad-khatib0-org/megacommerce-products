mod audit;
mod middleware;
mod product_create;
mod product_data;
mod router;

use std::{error::Error, net::SocketAddr, sync::Arc};

use megacommerce_proto::{products_service_server::ProductsServiceServer, Config as SharedConfig};
use tonic::{service::InterceptorLayer, transport::Server as GrpcServer};
use tower::ServiceBuilder;
use tracing::info;

use crate::{
  controller::middleware::context_middleware,
  models::errors::{ErrorType, InternalError},
  store::{cache::Cache, database::ProductsStore},
  utils::net::validate_url_target,
};

#[derive(Debug)]
pub struct Controller {
  pub(super) cfg: SharedConfig,
  pub(super) cache: Arc<Cache>,
  pub(super) store: Arc<dyn ProductsStore + Send + Sync>,
}

#[derive(Debug)]
pub struct ControllerArgs {
  pub cfg: SharedConfig,
  pub cache: Arc<Cache>,
  pub store: Arc<dyn ProductsStore + Send + Sync>,
}

impl Controller {
  pub fn new(args: ControllerArgs) -> Controller {
    Controller { cfg: args.cfg, cache: args.cache, store: args.store }
  }

  pub async fn run(self) -> Result<(), Box<dyn Error>> {
    let srv = self.cfg.services.as_ref().unwrap().clone();

    let url = srv.products_service_grpc_url();
    validate_url_target(url).map_err(|e| {
      Box::new(InternalError {
        err_type: ErrorType::Internal,
        err: Box::new(e),
        temp: false,
        msg: "failed to run products service server".into(),
        path: "products.controller.run".into(),
      })
    })?;

    let layer = ServiceBuilder::new()
      // .layer(InterceptorLayer::new(auth_middleware))
      .layer(InterceptorLayer::new(context_middleware))
      .into_inner();

    info!("products service server is running on: {}", url);
    GrpcServer::builder()
      .layer(layer)
      .add_service(ProductsServiceServer::new(self))
      .serve((url.parse::<SocketAddr>()).unwrap())
      .await?;

    Ok(())
  }
}
