use std::str::FromStr;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use megacommerce_proto::Product;
use megacommerce_shared::models::context::Context;
use megacommerce_shared::models::errors::{BoxedErr, ErrorType};
use megacommerce_shared::store::errors::DBError;
use serde_json::{to_value, Value};

use crate::store::database::dbstore::ProductsStoreImpl;

pub(super) async fn product_create(
  s: &ProductsStoreImpl,
  _: Arc<Context>,
  pro: &Product,
) -> Result<(), DBError> {
  let mk_err = |msg: &str, err: BoxedErr| DBError {
    err_type: ErrorType::JsonMarshal,
    err,
    msg: msg.into(),
    path: "products.store.product_create".into(),
    details: "".into(),
  };

  let price_decimal = BigDecimal::from_str(&pro.price)
    .map_err(|e| mk_err("invalid decimal string for price", Box::new(e)))?;

  let tags_json = to_value(&pro.tags)
    .map_err(|e| mk_err("failed to serialize the products tags", Box::new(e)))?;

  let metadata_json: Option<Value> = pro
    .metadata
    .as_ref()
    .map(|m| to_value(m))
    .transpose()
    .map_err(|e| mk_err("failed to serialize the products metadata", Box::new(e)))?;

  let db = &s.db;

  sqlx::query(
    r#"
        INSERT INTO products (
            id, user_id, sku, version, status,
            title, description, slug, price, currency_code,
            tags, metadata, ar_enabled,
            created_at, published_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12, $13,
            $14, $15, $16
        )
      "#,
  )
  .bind(&pro.id)
  .bind(&pro.user_id)
  .bind(&pro.sku)
  .bind(pro.version as i16)
  .bind(&pro.status)
  .bind(&pro.title)
  .bind(&pro.description)
  .bind(&pro.slug)
  .bind(price_decimal)
  .bind(&pro.currency_code)
  .bind(tags_json)
  .bind(metadata_json)
  .bind(pro.ar_enabled)
  .bind(pro.created_at as i64)
  .bind(pro.published_at.map(|t| t as i64))
  .bind(pro.updated_at.map(|t| t as i64))
  .execute(db.as_ref())
  .await
  .map_err(|e| mk_err("failed to insert a product", Box::new(e)))?;

  Ok(())
}
