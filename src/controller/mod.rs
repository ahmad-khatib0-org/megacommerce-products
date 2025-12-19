mod audit;
mod best_selling_products;
mod big_discount_products;
mod category_navbar;
mod helpers;
mod hero_products;
mod metrics;
mod newly_added_products;
mod product_create;
mod product_data;
mod product_details;
mod product_snapshot;
mod products_category;
mod products_list;
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

use self::metrics::MetricsCollector;
use crate::{
  otel::init_otel,
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
  pub metrics: Arc<MetricsCollector>,
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
    Controller {
      cfg: args.cfg,
      cache: args.cache,
      store: args.store,
      storage: args.storage,
      metrics: Arc::new(MetricsCollector::new(&prometheus::Registry::new()).unwrap()),
    }
  }

  pub async fn run(self) -> Result<(), Box<dyn Error>> {
    // Initialize OpenTelemetry
    let registry = init_otel("megacommerce-products").map_err(|_| {
      Box::new(InternalError {
        err_type: ErrorType::Internal,
        err: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "OTEL init failed")),
        temp: false,
        msg: "failed to initialize OTEL".into(),
        path: "products.controller.run".into(),
      })
    })?;

    // Initialize metrics with the registry
    let metrics = MetricsCollector::new(&registry).map_err(|e| {
      Box::new(InternalError {
        err_type: ErrorType::Internal,
        err: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)),
        temp: false,
        msg: "failed to initialize metrics".into(),
        path: "products.controller.run".into(),
      })
    })?;

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

    // Create controller with metrics
    let controller = Controller { metrics: Arc::new(metrics), ..self };

    let svc = ProductsServiceServer::new(controller);
    let layer_stack = ServiceBuilder::new().layer(InterceptorLayer::new(middleware_context));

    GrpcServer::builder()
      .layer(layer_stack)
      .add_service(svc)
      .serve(url.parse::<SocketAddr>().unwrap())
      .await?;

    Ok(())
  }
}
