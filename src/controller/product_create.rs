use std::sync::Arc;

use megacommerce_proto::{
  product_create_response::Response::{Data, Error as ResError},
  Empty, ProductCreateRequest, ProductCreateResponse,
};
use tonic::{Request, Response, Status};

use crate::{
  controller::Controller,
  models::{
    audit::{AuditRecord, EventName::ProductCreate, EventParameterKey, EventStatus::Fail},
    context::Context,
    errors::AppError,
    product_create::{
      products_create_auditable, products_create_is_valid, products_create_pre_save,
    },
  },
};

pub(super) async fn product_create(
  c: &Controller,
  req: Request<ProductCreateRequest>,
) -> Result<Response<ProductCreateResponse>, Status> {
  let ctx = req.extensions().get::<Arc<Context>>().cloned().unwrap();
  let pro = req.into_inner();
  let path = "products.controller.product_create";
  let return_err =
    |e: AppError| Response::new(ProductCreateResponse { response: Some(ResError(e.to_proto())) });

  let mut audit = AuditRecord::new(ctx.clone(), ProductCreate, Fail);
  audit.set_event_parameter(EventParameterKey::ProductCreate, products_create_auditable(&pro));

  if let Err(err) = products_create_is_valid(ctx.clone(), &pro, &c.cache.tags_as_ref()) {
    return Ok(return_err(err));
  }

  let pro_db = products_create_pre_save(ctx.clone(), &pro);
  if pro_db.is_err() {
    return Ok(return_err(pro_db.unwrap_err().to_internal(ctx.clone(), path.into())));
  }

  if let Err(err) = c.store.product_create(ctx.clone(), &pro_db.unwrap()).await {
    return Ok(return_err(err.to_app_error_internal(ctx.clone(), path.into())));
  }

  audit.success();
  Ok(Response::new(ProductCreateResponse { response: Some(Data(Empty {})) }))
}
