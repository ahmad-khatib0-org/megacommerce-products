use std::sync::Arc;

use megacommerce_proto::{PaginationRequest, PaginationResponse};
use megacommerce_shared::models::{context::Context, errors::AppError};
use tonic::Code;
use ulid::Ulid;

pub(super) fn check_last_id(
  ctx: Arc<Context>,
  _where: &str,
  pagination: &Option<PaginationRequest>,
) -> Result<(), AppError> {
  let mk_err =
    |id: &str| AppError::new(ctx, _where, id, None, "", Code::InvalidArgument.into(), None);

  if pagination.is_none() {
    return Err(mk_err("request.pagination.invalid"));
  }

  let pagination = pagination.as_ref().unwrap();
  let last_id = pagination.last_id();
  let page = pagination.page();

  if page > 1 && last_id == "" {
    return Err(mk_err("request.last_id.missing"));
  }
  if page > 1 {
    Ulid::from_string(last_id).map_err(|_| mk_err("request.last_id.invalid"))?;
  }
  Ok(())
}

pub(super) fn build_pagination_response(
  pr: &PaginationRequest,
  items_count: usize,
) -> PaginationResponse {
  PaginationResponse {
    has_previous: Some(pr.page() != 1),
    has_next: Some(pr.page_size() == items_count as u32),
    ..Default::default()
  }
}
