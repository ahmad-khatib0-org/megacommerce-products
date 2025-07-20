use std::sync::Arc;

use tonic::Code;
use ulid::Ulid;

use crate::models::{context::Context, errors::AppError};

pub(super) fn check_last_id(
  ctx: Arc<Context>,
  _where: &str,
  page: &u32,
  last_id: &str,
) -> Result<(), AppError> {
  let mk_err =
    |id: &str| AppError::new(ctx, _where, id, None, "", Some(Code::InvalidArgument.into()), None);

  if *page > 1 && last_id == "" {
    return Err(mk_err("request.last_id.missing"));
  }

  if *page > 1 {
    Ulid::from_string(last_id).map_err(|_| mk_err("request.last_id.invalid"))?;
  }

  Ok(())
}
