use std::sync::Arc;

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
  let mk_err = |msg: &str, err: BoxedErr, typ: Option<ErrorType>| DBError {
    err_type: typ.unwrap_or(ErrorType::JsonMarshal),
    err,
    msg: msg.into(),
    path: "products.store.product_create".into(),
    details: "".into(),
  };

  let bullet_points: Value = to_value(&pro.bullet_points)
    .map_err(|e| mk_err("failed to serialize the product's bullet_points", Box::new(e), None))?;

  let details: Value = to_value(pro.details.as_ref())
    .map_err(|e| mk_err("failed to serialize the product's details", Box::new(e), None))?;
  let media: Value = to_value(pro.media.as_ref())
    .map_err(|e| mk_err("failed to serialize the product's media", Box::new(e), None))?;
  let offer: Value = to_value(pro.offer.as_ref())
    .map_err(|e| mk_err("failed to serialize the product's offer", Box::new(e), None))?;
  let safety: Value = to_value(pro.safety.as_ref())
    .map_err(|e| mk_err("failed to serialize the product's safety", Box::new(e), None))?;

  let tags = to_value(&pro.tags)
    .map_err(|e| mk_err("failed to serialize the product's tags", Box::new(e), None))?;
  let metadata: Option<Value> = pro
    .metadata
    .as_ref()
    .map(|m| to_value(m))
    .transpose()
    .map_err(|e| mk_err("failed to serialize the products metadata", Box::new(e), None))?;

  let db = &*s.db.get().await;

  sqlx::query(
    r#"
    INSERT INTO products (
        id, user_id, title, category, subcategory, has_variations, brand_name,
        has_brand_name, product_id, has_product_id, product_id_type, description, 
        bullet_points, currency_code, fulfillment_type, processing_time, details, 
        media, offer, safety, tags, metadata, ar_enabled, slug, status, version, 
        schema_version, created_at, published_at, updated_at
    ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
        $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
        $21, $22, $23, $24, $25, $26, $27, $28, $29, $30
    )
  "#,
  )
  .bind(&pro.id)
  .bind(&pro.user_id)
  .bind(&pro.title)
  .bind(&pro.category)
  .bind(&pro.subcategory)
  .bind(&pro.has_variations)
  .bind(&pro.brand_name)
  .bind(&pro.has_brand_name)
  .bind(&pro.product_id)
  .bind(&pro.has_product_id)
  .bind(&pro.product_id_type)
  .bind(&pro.description)
  .bind(&bullet_points)
  .bind(&pro.currency_code)
  .bind(&pro.fulfillment_type)
  .bind(pro.processing_time as i64)
  .bind(&details)
  .bind(&media)
  .bind(&offer)
  .bind(&safety)
  .bind(&tags)
  .bind(&metadata)
  .bind(pro.ar_enabled)
  .bind(&pro.slug)
  .bind(&pro.status)
  .bind(pro.version as i32)
  .bind(pro.schema_version as i32)
  .bind(pro.created_at as i64)
  .bind(pro.published_at.map(|t| t as i64))
  .bind(pro.updated_at.map(|t| t as i64))
  .execute(db)
  .await
  .map_err(|e| mk_err("failed to insert a product", Box::new(e), Some(ErrorType::DBInsertError)))?;

  Ok(())
}
