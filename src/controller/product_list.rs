use std::sync::Arc;

use megacommerce_proto::{
  product_list_response::Response::{Data, Error as ResError},
  ProductListRequest, ProductListResponse, ProductListResponseData,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, OptionalParams, MSG_ID_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::{helpers::check_last_id, Controller};

pub(super) async fn product_list(
  c: &Controller,
  request: Request<ProductListRequest>,
) -> Result<Response<ProductListResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();

  let w = "products.controller.product_list";
  let return_err =
    |e: AppError| Response::new(ProductListResponse { response: Some(ResError(e.to_proto())) });
  let ie = |id: &str, p: OptionalParams, err: Option<AppErrorErrors>, code: Option<Code>| {
    AppError::new(ctx.clone(), w, id, p, "", code.unwrap_or(Code::Internal).into(), err)
  };

  let mut res = ProductListResponseData { ..Default::default() };
  if let Err(err) = check_last_id(ctx.clone(), "", &req.page, &req.last_id) {
    return Ok(return_err(err));
  }

  let res_db = c.store.product_list(ctx.clone(), &req).await;
  if let Err(err) = res_db {
    return Ok(return_err(ie(
      MSG_ID_ERR_INTERNAL,
      None,
      Some(AppErrorErrors { err: Some(Box::new(err)), ..Default::default() }),
      None,
    )));
  } else {
    res.data = res_db.unwrap()
  }

  Ok(Response::new(ProductListResponse { response: Some(Data(res)) }))
}
