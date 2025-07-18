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

  if req.get_all_categories.unwrap_or(false) {
    res.categories = Some(c.cache.categories());
  }

  if req.get_tags.unwrap_or(false) {
    res.tags = Some(ProductTags { tags: c.cache.tags() });
  }

  if req.get_category_data.unwrap_or(false) {
    let cat_name = req.category_name.unwrap_or_default();
    if cat_name == "".to_string() {
      return Ok(return_err(mk_err("categories.missing_name.error", None, None)));
    }
    if let Some(cat_data) = c.cache.category_data(&cat_name) {
      res.category_data = Some(cat_data);
    } else {
      return Ok(return_err(mk_err("categories.not_found.error", None, None)));
    }
  }

  Ok(Response::new(ProductDataResponse { response: Some(Data(res)) }))
}
