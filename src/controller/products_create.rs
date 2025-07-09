use std::sync::Arc;

use megacommerce_proto::{
  product_create_response::Response::{Data, Error as ResError},
  products_service_server::ProductsService,
  ProductCreateRequest, ProductCreateResponse,
};
use tonic::{Request, Response, Status};

use crate::{
  controller::Controller,
  models::{
    audit::{AuditRecord, EventName::ProductCreate, EventParameterKey, EventStatus::Fail},
    context::Context,
    errors::AppError,
    products_create::{
      products_create_auditable, products_create_is_valid, products_create_pre_save,
    },
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
    let path = "products.controller.product_create";
    let return_err =
      |e: AppError| Response::new(ProductCreateResponse { response: Some(ResError(e.to_proto())) });

    let mut audit = AuditRecord::new(ctx.clone(), ProductCreate, Fail);
    audit.set_event_parameter(EventParameterKey::ProductCreate, products_create_auditable(&pro));

    if let Err(err) = products_create_is_valid(ctx.clone(), &pro, self.cache.tags()) {
      return Ok(return_err(err));
    }

    let pro_db = products_create_pre_save(ctx.clone(), &pro);
    if pro_db.is_err() {
      return Ok(return_err(pro_db.unwrap_err().to_internal(ctx.clone(), path.into())));
    }

    if let Err(err) = self.store.product_create(ctx.clone(), &pro_db.unwrap()).await {
      return Ok(return_err(err.to_app_error_internal(ctx.clone(), path.into())));
    }

    audit.success();
    Ok(Response::new(ProductCreateResponse { response: Some(Data(())) }))
  }
}
