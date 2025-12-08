use std::sync::Arc;

use megacommerce_proto::{
  category_navbar_response::Response::{Data, Error},
  CategoryNavbarRequest, CategoryNavbarResponse,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, MSG_ID_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::Controller;

pub async fn category_navbar(
  c: &Controller,
  req: Request<CategoryNavbarRequest>,
) -> Result<Response<CategoryNavbarResponse>, Status> {
  let ctx = req.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req_data = req.into_inner();
  let path = "products.controller.category_navbar";
  let return_err = |e: AppError| {
    return Response::new(CategoryNavbarResponse { response: Some(Error(e.to_proto())) });
  };

  let ie = |err: BoxedErr, id: &str, code: Option<Code>| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, id, None, "", code.unwrap_or(Code::Internal).into(), errors)
  };

  let result =
    c.store.category_navbar(ctx.clone(), &req_data.category_id, &req_data.subcategory_id).await;

  match result {
    Ok(data) => {
      return Ok(Response::new(CategoryNavbarResponse { response: Some(Data(data)) }));
    }
    Err(err) => {
      if err.err_type == ErrorType::NoRows {
        let id = "categories.not_found.error";
        return Ok(return_err(ie(Box::new(err), id, Some(Code::NotFound))));
      }
      return Ok(return_err(ie(Box::new(err), MSG_ID_ERR_INTERNAL, Some(Code::Internal))));
    }
  }
}
