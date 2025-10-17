use std::sync::Arc;

use megacommerce_proto::{
  product_data_response::Response::{Data, Error as ResError},
  ProductDataRequest, ProductDataResponse, ProductDataResponseData, ProductTags,
};
use tonic::{Code, Request, Response, Status};

use crate::{
  controller::Controller,
  models::{
    context::Context,
    errors::{AppError, OptionalError, OptionalParams},
  },
};

pub(super) async fn product_data(
  c: &Controller,
  request: Request<ProductDataRequest>,
) -> Result<Response<ProductDataResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();
  let lang = ctx.accept_language();
  let return_err =
    |e: AppError| Response::new(ProductDataResponse { response: Some(ResError(e.to_proto())) });
  let mk_err = |id: &str, p: OptionalParams, err: OptionalError| {
    AppError::new(
      ctx.clone(),
      "products.controller.product_data",
      id,
      p,
      "",
      Some(Code::InvalidArgument.into()),
      err,
    )
  };

  let mut res = ProductDataResponseData { ..Default::default() };

  if req.get_tags.unwrap_or(false) {
    res.tags = Some(ProductTags { tags: c.cache.tags() });
  }

  if !req.subcategory.is_none() {
    let sub = req.subcategory.as_ref().unwrap();
    let cat_name = &sub.category;
    let sub_name = &sub.subcategory;

    println!("the categories: {} {} {}", cat_name, sub_name, lang);
    if cat_name.is_empty() || sub_name.is_empty() {
      return Ok(return_err(mk_err("categories.missing_name.error", None, None)));
    }
    if let Some(sub) = c.cache.subcategory_data(cat_name, sub_name, lang) {
      res.subcategory = Some(sub);
    } else {
      return Ok(return_err(mk_err("categories.not_found.error", None, None)));
    }
  }

  Ok(Response::new(ProductDataResponse { response: Some(Data(res)) }))
}
