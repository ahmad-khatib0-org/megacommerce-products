mod audit;
mod best_selling_products;
mod big_discount_products;
mod helpers;
mod hero_products;
mod newly_added_products;
mod product_create;
mod product_data;
mod product_snapshot;
mod products_to_like;
mod router;

use std::{error::Error, net::SocketAddr, sync::Arc};

use megacommerce_proto::{products_service_server::ProductsServiceServer, Config as SharedConfig};
use megacommerce_shared::{
  models::{
    errors::{ErrorType, InternalError},
    r_lock::RLock,
  },
  utils::middleware::middleware_context,
};
use tonic::{service::InterceptorLayer, transport::Server as GrpcServer};
use tower::ServiceBuilder;

use crate::{
  server::object_storage::ObjectStorage,
  store::{cache::Cache, database::ProductsStore},
  utils::net::validate_url_target,
};

#[derive(Debug)]
pub struct Controller {
  pub(super) cfg: RLock<SharedConfig>,
  pub(super) cache: Arc<Cache>,
  pub(super) store: Arc<dyn ProductsStore + Send + Sync>,
  pub storage: RLock<ObjectStorage>,
}

#[derive(Debug)]
pub struct ControllerArgs {
  pub cfg: RLock<SharedConfig>,
  pub storage: RLock<ObjectStorage>,
  pub cache: Arc<Cache>,
  pub store: Arc<dyn ProductsStore + Send + Sync>,
}

impl Controller {
  pub fn new(args: ControllerArgs) -> Controller {
    Controller { cfg: args.cfg, cache: args.cache, store: args.store, storage: args.storage }
  }

  pub async fn run(self) -> Result<(), Box<dyn Error>> {
    let srv = self.cfg.get().await.services.as_ref().unwrap().clone();

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

    let svc = ProductsServiceServer::new(self);
    let layer_stack = ServiceBuilder::new().layer(InterceptorLayer::new(middleware_context));

    GrpcServer::builder()
      .layer(layer_stack)
      .add_service(svc)
      .serve(url.parse::<SocketAddr>().unwrap())
      .await?;

    Ok(())
  }
}
