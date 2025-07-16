use std::{
  collections::{HashMap, HashSet},
  fmt::Display,
  sync::{Arc, RwLockReadGuard},
};

use megacommerce_proto::{Product, ProductCreateRequest, ProductCreateTag, ProductTag};
use serde_json::{json, Value};
use tonic::Code;

use crate::{
  data::currencies::CURRENCY_LIST,
  models::{
    context::Context,
    errors::AppError,
    products::{
      PRODUCT_DESCRIPTION_MAX_LENGTH, PRODUCT_DESCRIPTION_MIN_LENGTH, PRODUCT_SKU_MAX_LENGTH,
      PRODUCT_SKU_MIN_LENGTH, PRODUCT_TITLE_MAX_LENGTH, PRODUCT_TITLE_MIN_LENGTH,
    },
  },
  utils::{slug::Slug, time::time_get_millis},
};

use super::products::ProductStatus;

pub fn products_create_is_valid(
  ctx: Arc<Context>,
  product: &ProductCreateRequest,
  existing_tags: RwLockReadGuard<'_, Vec<ProductTag>>,
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

  if let Err(tag) = check_if_tags_exist(existing_tags, tags) {
    let p = HashMap::from([("Tag".to_string(), Value::String(tag.name.unwrap_or_default()))]);
    return Err(error_builder(ctx, "tags.not_exists", title, Some(p)));
  }

  let price_err = || {
    error_builder(
      ctx.clone(),
      "price.invalid",
      price,
      Some(HashMap::from([("Price".to_string(), Value::String(price.clone()))])),
    )
  };
  let parsed_price = price.parse::<f64>().map_err(|_| price_err())?;
  if parsed_price <= 0.0 {
    return Err(price_err());
  }

  Ok(())
}

pub fn products_create_pre_save(
  ctx: Arc<Context>,
  pro: &ProductCreateRequest,
) -> Result<Product, AppError> {
  let id = ulid::Ulid::new().to_string();
  let slug = Slug::default().generate_slug(&pro.title);
  let tags = pro
    .tags
    .clone()
    .iter()
    .map(|t| {
      let id = if t.id.is_some() { t.id } else { None };
      let name = if t.name.is_some() { t.name.clone() } else { None };
      ProductTag { id, name }
    })
    .collect();

  Ok(Product {
    id,
    user_id: ctx.session().user_id().to_string(),
    sku: pro.sku.clone(),
    version: 1,
    status: ProductStatus::Pending.as_string(),
    title: pro.title.clone(),
    description: pro.description.clone(),
    slug: slug,
    price: pro.price.clone(),
    currency_code: pro.currency_code.clone(),
    tags,
    metadata: None,
    ar_enabled: pro.ar_enabled,
    created_at: time_get_millis(),
    published_at: None,
    updated_at: None,
  })
}

pub fn products_create_auditable(p: &ProductCreateRequest) -> Value {
  let tags: Vec<Value> = p.tags.iter().map(|t| json!({"id": t.id ,  "name": t.name})).collect();

  json!({
    "title": p.title,
    "description": p.description,
    "sku": p.sku,
    "price": p.price,
    "currency_code": p.currency_code,
    "tags": tags,
    "ar_enabled" : p.ar_enabled,
  })
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

fn check_if_tags_exist(
  existing: RwLockReadGuard<'_, Vec<ProductTag>>,
  incoming: &Vec<ProductCreateTag>,
) -> Result<(), ProductCreateTag> {
  let existing_tags: HashSet<u32> = existing.iter().filter_map(|t| t.id).collect();
  for tag in incoming {
    if let Some(id) = tag.id {
      if !existing_tags.contains(&id) {
        return Err(tag.clone());
      }
    }
  }

  Ok(())
}
