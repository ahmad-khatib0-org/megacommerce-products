use std::{
  io::{Error, ErrorKind},
  sync::Arc,
};

use megacommerce_proto::{
  product_details_response::Response::{Data, Error as ResError},
  ProductDetailsRequest, ProductDetailsResponse,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, MSG_ID_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::{helpers::is_valid_ulid, Controller};

pub async fn product_details(
  c: &Controller,
  request: Request<ProductDetailsRequest>,
) -> Result<Response<ProductDetailsResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();
  let path = "products.controller.product_details";
  let return_err = |e: AppError| {
    return Response::new(ProductDetailsResponse { response: Some(ResError(e.to_proto())) });
  };
  let ie = |err: BoxedErr, id: &str, code: Option<Code>| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, id, None, "", code.unwrap_or(Code::Internal).into(), errors)
  };

  let not_found = |err: Option<BoxedErr>| {
    return_err(ie(
      err.unwrap_or(Box::new(Error::new(ErrorKind::NotFound, "the requsted product is not found"))),
      "error.not_found",
      Some(Code::NotFound),
    ))
  };

  if !is_valid_ulid(&req.product_id) {
    return Ok(not_found(None));
  }

  let product = c.store.product_details(ctx.clone(), &req.product_id).await;
  if product.is_err() {
    let err = product.unwrap_err();
    match err.err_type {
      ErrorType::NoRows => return Ok(not_found(Some(Box::new(err)))),
      _ => return Ok(return_err(ie(Box::new(err), MSG_ID_ERR_INTERNAL, None))),
    }
  }

  Ok(Response::new(ProductDetailsResponse { response: Some(Data(product.unwrap())) }))
}
