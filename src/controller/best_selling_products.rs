use std::sync::Arc;

use megacommerce_proto::{
  best_selling_products_response::Response::{Data, Error},
  BestSellingProductsRequest, BestSellingProductsResponse,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, MSG_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::Controller;

pub(super) async fn best_selling_products(
  c: &Controller,
  request: Request<BestSellingProductsRequest>,
) -> Result<Response<BestSellingProductsResponse>, Status> {
  let start = std::time::Instant::now();
  c.metrics.best_selling_products_total.inc();
  
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let w = "products.controller.best_selling_products";
  let return_err = |e: AppError| {
    Response::new(BestSellingProductsResponse { response: Some(Error(e.to_proto())) })
  };
  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), w, MSG_ERR_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  let products = c.store.best_selling_products(ctx.clone()).await;
  if products.is_err() {
    c.metrics.record_best_selling_products_error();
    return Ok(return_err(ie(Box::new(products.unwrap_err()))));
  }

  let duration = start.elapsed().as_secs_f64();
  c.metrics.record_best_selling_products_success(duration);

  Ok(Response::new(BestSellingProductsResponse {
    response: Some(Data(megacommerce_proto::BestSellingProductsResponseData {
      products: products.unwrap(),
    })),
  }))
}
