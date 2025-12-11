use std::sync::Arc;

use megacommerce_proto::{
  products_category_response::Response::{Data, Error},
  ProductsCategoryRequest, ProductsCategoryResponse, ProductsCategoryResponseData, SortDirection,
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

pub(super) async fn products_category(
  c: &Controller,
  request: Request<ProductsCategoryRequest>,
) -> Result<Response<ProductsCategoryResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let req = request.into_inner();

  let path = "products.controller.products_category";
  let return_err =
    |e: AppError| Response::new(ProductsCategoryResponse { response: Some(Error(e.to_proto())) });

  let ie = |err: BoxedErr| {
    println!("{}", err);
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, MSG_ID_ERR_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  // Validate pagination
  if let Err(err) = check_last_id(ctx.clone(), path, &req.pagination) {
    return Ok(return_err(err));
  }

  let pagination = req.pagination.unwrap();
  let limit = 20i64;
  let page = pagination.page();
  let last_id = pagination.last_id();

  // Extract sorting info from pagination.sort_by
  let (sort_by, sort_direction) = if !pagination.sort_by.is_empty() {
    if let Some(first_sort) = pagination.sort_by.first() {
      let direction = match first_sort.direction() {
        SortDirection::Asc => Some("asc"),
        SortDirection::Desc => Some("desc"),
        _ => None,
      };
      (Some(first_sort.name.as_str()), direction)
    } else {
      (None, None)
    }
  } else {
    (None, None)
  };

  let result = c
    .store
    .products_category(
      ctx.clone(),
      &req.category_id,
      &req.subcategory_ids,
      page,
      &last_id,
      limit,
      sort_by,
      sort_direction,
    )
    .await;

  match result {
    Ok(products) => {
      let pagination_response = build_pagination_response(&pagination, products.iter().count());
      Ok(Response::new(ProductsCategoryResponse {
        response: Some(Data(ProductsCategoryResponseData {
          products,
          pagination: Some(pagination_response),
        })),
      }))
    }
    Err(err) => Ok(return_err(ie(Box::new(err)))),
  }
}
