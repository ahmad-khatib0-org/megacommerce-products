use std::sync::Arc;

use megacommerce_proto::{
  big_discount_products_response::Response::{Data, Error},
  BigDiscountProductsRequest, BigDiscountProductsResponse,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, MSG_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::Controller;

pub(super) async fn big_discount_products(
  c: &Controller,
  request: Request<BigDiscountProductsRequest>, // Changed
) -> Result<Response<BigDiscountProductsResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let w = "products.controller.big_discount_products";
  let return_err = |e: AppError| {
    Response::new(BigDiscountProductsResponse { response: Some(Error(e.to_proto())) })
  };
  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), w, MSG_ERR_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  let products = c.store.big_discount_products(ctx.clone()).await;
  if products.is_err() {
    return Ok(return_err(ie(Box::new(products.unwrap_err()))));
  }

  Ok(Response::new(BigDiscountProductsResponse {
    response: Some(Data(megacommerce_proto::BigDiscountProductsResponseData {
      products: products.unwrap(),
    })),
  }))
}
