use std::sync::Arc;

use megacommerce_proto::{
  products_service_server::ProductsService, ProductCreateRequest, ProductCreateResponse,
};
use tonic::{Request, Response, Status};

use crate::{
  controller::main::Controller,
  models::{
    audit::{AuditRecord, EventName::ProductCreate, EventParameterKey, EventStatus::Fail},
    context::Context,
    products_create::products_create_create_auditable,
  },
};

#[tonic::async_trait]
impl ProductsService for Controller {
  async fn product_create(
    &self,
    req: Request<ProductCreateRequest>,
  ) -> Result<Response<ProductCreateResponse>, Status> {
    let ctx = req.extensions().get::<Arc<Context>>().cloned().unwrap();
    let pro = req.into_inner();

    let mut audit = AuditRecord::new(ctx, ProductCreate, Fail);
    audit.set_event_parameter(
      EventParameterKey::ProductCreate,
      products_create_create_auditable(&pro),
    );
    println!("{}", &audit);

    let response = ProductCreateResponse {};
    Ok(Response::new(response))
  }
}
