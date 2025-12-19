use std::sync::Arc;

use megacommerce_proto::{
  product_data_response::Response::{Data, Error as ResError},
  ProductDataRequest, ProductDataResponse, ProductDataResponseData, ProductTags,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, OptionalParams},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::Controller;

pub(super) async fn product_data(
  c: &Controller,
  request: Request<ProductDataRequest>,
) -> Result<Response<ProductDataResponse>, Status> {
  let start = std::time::Instant::now();
  c.metrics.product_data_total.inc();
  
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();
  let lang = ctx.accept_language();
  let return_err = |e: AppError| {
    c.metrics.record_product_data_error();
    Response::new(ProductDataResponse { response: Some(ResError(e.to_proto())) })
  };
  let mk_err = |id: &str, p: OptionalParams, err: Option<AppErrorErrors>| {
    let path = "products.controller.product_data";
    AppError::new(ctx.clone(), path, id, p, "", Code::InvalidArgument.into(), err)
  };

  let mut res = ProductDataResponseData { ..Default::default() };

  if req.get_tags.unwrap_or(false) {
    res.tags = Some(ProductTags { tags: c.cache.tags() });
  }

  if !req.subcategory.is_none() {
    let sub = req.subcategory.as_ref().unwrap();
    let cat_name = &sub.category;
    let sub_name = &sub.subcategory;

    if cat_name.is_empty() || sub_name.is_empty() {
      return Ok(return_err(mk_err("categories.missing_name.error", None, None)));
    }
    if let Some(sub) = c.cache.subcategory_data(cat_name, sub_name, lang) {
      res.subcategory = Some(sub);
    } else {
      return Ok(return_err(mk_err("categories.not_found.error", None, None)));
    }
  }

  let duration = start.elapsed().as_secs_f64();
  c.metrics.record_product_data_success(duration);
  Ok(Response::new(ProductDataResponse { response: Some(Data(res)) }))
}
