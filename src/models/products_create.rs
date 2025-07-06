use std::{collections::HashMap, fmt::Display, sync::Arc};

use megacommerce_proto::{ProductCreateRequest, ProductTag};
use serde_json::{Number, Value};
use tonic::Code;

use crate::{
  data::currencies::CURRENCY_LIST,
  models::{context::Context, errors::AppError},
};

static PRODUCT_TITLE_MIN_LENGTH: usize = 5;
static PRODUCT_TITLE_MAX_LENGTH: usize = 250;
static PRODUCT_DESCRIPTION_MIN_LENGTH: usize = 20;
static PRODUCT_DESCRIPTION_MAX_LENGTH: usize = 1024;
static PRODUCT_SKU_MIN_LENGTH: usize = 3;
static PRODUCT_SKU_MAX_LENGTH: usize = 60;

pub fn products_create_is_valid(
  ctx: Arc<Context>,
  product: &ProductCreateRequest,
  existing_tags: &Vec<ProductTag>,
) -> Result<(), AppError> {
  let ProductCreateRequest { title, description, currency_code, sku, tags, price, ar_enabled } =
    product;

  if title.chars().count() < PRODUCT_TITLE_MIN_LENGTH
    || title.chars().count() > PRODUCT_TITLE_MAX_LENGTH
  {
    let p = HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_TITLE_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_TITLE_MAX_LENGTH.into())),
    ]);
    return Err(error_builder(ctx, "title", title, Some(p)));
  }

  if description.chars().count() < PRODUCT_DESCRIPTION_MIN_LENGTH
    || description.chars().count() > PRODUCT_DESCRIPTION_MAX_LENGTH
  {
    let p = HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_DESCRIPTION_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_DESCRIPTION_MAX_LENGTH.into())),
    ]);
    return Err(error_builder(ctx, "description", title, Some(p)));
  }

  let mut valid_currency = false;
  for c in CURRENCY_LIST {
    if c.code == currency_code {
      valid_currency = true;
      break;
    }
  }
  if !valid_currency {
    return Err(error_builder(ctx, "currency_code", currency_code, None));
  }

  if sku.chars().count() < PRODUCT_SKU_MIN_LENGTH || sku.chars().count() > PRODUCT_SKU_MAX_LENGTH {
    let p = HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_SKU_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_SKU_MAX_LENGTH.into())),
    ]);
    return Err(error_builder(ctx, "sku", title, Some(p)));
  }

  for tag in tags {
    if !existing_tags.contains(tag) {
      let p = HashMap::from([("Tag".to_string(), Value::String(tag.name.clone()))]);
      return Err(error_builder(ctx, "tags.not_exists", title, Some(p)));
    }
  }

  if *price <= 0 {
    let p = HashMap::from([("Price".to_string(), Value::Number(Number::from(*price)))]);
    return Err(error_builder(ctx, "tags.not_exists", title, Some(p)));
  }

  Ok(())
}

fn error_builder<T: Display>(
  ctx: Arc<Context>,
  field_name: &str,
  field_value: T,
  params: Option<HashMap<String, Value>>,
) -> AppError {
  let id = format!("products.{}.error", field_name);
  let details = format!("{}={}", field_name, field_value);
  AppError::new(
    ctx,
    "products.models.products_create.products_create_is_valid",
    id,
    params,
    details,
    Some(Code::InvalidArgument.into()),
    None,
  )
}
