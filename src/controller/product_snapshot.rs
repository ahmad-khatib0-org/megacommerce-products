use std::sync::Arc;

use megacommerce_proto::{
  product_snapshot_response::Response::{Data as ResData, Error as ResError},
  ProductSnapshotRequest, ProductSnapshotResponse,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, OptionalParams},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::Controller;

pub async fn product_snapshot(
  c: &Controller,
  req: Request<ProductSnapshotRequest>,
) -> Result<Response<ProductSnapshotResponse>, Status> {
  let start = std::time::Instant::now();
  c.metrics.product_snapshot_total.inc();
  
  let ctx = req.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = req.into_inner();
  let path = "products.controller.product_snapshot";
  let return_err = |e: AppError| {
    return Response::new(ProductSnapshotResponse { response: Some(ResError(e.to_proto())) });
  };

  let ie = |id: &str, p: OptionalParams, err: Option<AppErrorErrors>, code: Option<Code>| {
    AppError::new(ctx.clone(), path, id, p, "", code.unwrap_or(Code::Internal).into(), err)
  };

  let product_snapshot = c.store.product_snapshot(ctx.clone(), &req).await;
  if product_snapshot.is_err() {
    c.metrics.record_product_snapshot_error();
    let unwrapped = product_snapshot.unwrap_err();
    match unwrapped.err_type {
      ErrorType::NoRows => {
        let error =
          Some(AppErrorErrors { err: Some(Box::new(unwrapped) as BoxedErr), ..Default::default() });
        return Ok(return_err(ie("products.not_found.error", None, error, Some(Code::NotFound))));
      }
      _ => return Ok(return_err(unwrapped.to_app_error_internal(ctx, path.to_string()))),
    }
  }

  let duration = start.elapsed().as_secs_f64();
  c.metrics.record_product_snapshot_success(duration);
  Ok(Response::new(ProductSnapshotResponse { response: Some(ResData(product_snapshot.unwrap())) }))
}
