use std::sync::Arc;

use megacommerce_proto::{
  products_to_like_response::Response::{Data, Error},
  ProductsToLikeRequest, ProductsToLikeResponse, ProductsToLikeResponseData,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, MSG_ID_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::{
  helpers::{build_pagination_response, check_last_id},
  Controller,
};

pub(super) async fn products_to_like(
  c: &Controller,
  request: Request<ProductsToLikeRequest>,
) -> Result<Response<ProductsToLikeResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();

  let w = "products.controller.products_to_like";
  let return_err =
    |e: AppError| Response::new(ProductsToLikeResponse { response: Some(Error(e.to_proto())) });
  let ie = |err: BoxedErr| {
    println!("{}", err);
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), w, MSG_ID_ERR_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = check_last_id(ctx.clone(), w, &req.pagination) {
    return Ok(return_err(err));
  }

  let pagination = &mut req.pagination.unwrap();
  pagination.page_size = Some(20);
  let limit = pagination.page_size() as i64;
  let result =
    c.store.products_to_like(ctx.clone(), pagination.page(), pagination.last_id(), limit).await;

  if let Err(err) = result {
    return Ok(return_err(ie(Box::new(err))));
  } else {
    let products = result.unwrap();
    Ok(Response::new(ProductsToLikeResponse {
      response: Some(Data(ProductsToLikeResponseData {
        pagination: Some(build_pagination_response(pagination, products.iter().count())),
        products,
      })),
    }))
  }
}
