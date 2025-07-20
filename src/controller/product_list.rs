use std::sync::Arc;

use megacommerce_proto::{
  product_list_response::Response::{Data, Error as ResError},
  ProductListRequest, ProductListResponse, ProductListResponseData,
};
use tonic::{Code, Request, Response, Status};

use crate::{
  controller::{helpers::check_last_id, Controller},
  models::{
    context::Context,
    errors::{AppError, OptionalError, OptionalParams, MSG_ID_ERR_INTERNAL},
  },
};

pub(super) async fn product_list(
  c: &Controller,
  request: Request<ProductListRequest>,
) -> Result<Response<ProductListResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();

  let w = "products.controller.product_list";
  let return_err =
    |e: AppError| Response::new(ProductListResponse { response: Some(ResError(e.to_proto())) });
  let mk_err = |id: &str, p: OptionalParams, err: OptionalError, code: Option<Code>| {
    AppError::new(ctx.clone(), w, id, p, "", Some(code.unwrap_or(Code::Internal).into()), err)
  };

  let mut res = ProductListResponseData { ..Default::default() };
  if let Err(err) = check_last_id(ctx.clone(), "", &req.page, &req.last_id) {
    return Ok(return_err(err));
  }

  let res_db = c.store.product_list(ctx.clone(), &req).await;
  if let Err(err) = res_db {
    return Ok(return_err(mk_err(MSG_ID_ERR_INTERNAL, None, Some(Box::new(err)), None)));
  } else {
    res.data = res_db.unwrap()
  }

  Ok(Response::new(ProductListResponse { response: Some(Data(res)) }))
}
