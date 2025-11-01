use std::sync::Arc;

use megacommerce_proto::{
  product_create_response::Response::{Data as ResData, Error as ResError},
  ProductCreateRequest, ProductCreateRequestIdentity, ProductCreateResponse, SuccessResponseData,
};
use megacommerce_shared::models::{context::Context, errors::AppError, translate::tr};
use tonic::{Request, Response, Status};

use crate::{
  controller::Controller,
  models::{
    audit::{AuditRecord, EventName::ProductCreate, EventParameterKey, EventStatus::Fail},
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
  let lang = ctx.accept_language();
  let path = "products.controller.product_create";
  let return_err =
    |e: AppError| Response::new(ProductCreateResponse { response: Some(ResError(e.to_proto())) });

  let mut audit = AuditRecord::new(ctx.clone(), ProductCreate, Fail);
  audit.set_event_parameter(EventParameterKey::ProductCreate, products_create_auditable(&pro));

  let identity = pro.identity.clone().unwrap_or(ProductCreateRequestIdentity::default());
  let subcategory_data = c.cache.subcategory_data(&identity.category, &identity.subcategory, lang);

  if let Err(err) = products_create_is_valid(ctx.clone(), &pro, subcategory_data) {
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

  let message = tr::<()>(lang, "products.create.successfully", None).unwrap_or_default();

  Ok(Response::new(ProductCreateResponse {
    response: Some(ResData(SuccessResponseData { message: Some(message), ..Default::default() })),
  }))
}
