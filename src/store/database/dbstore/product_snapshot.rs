use std::sync::Arc;

use megacommerce_proto::{ProductOffer, ProductSnapshot, ProductSnapshotRequest};
use megacommerce_shared::{
  models::{context::Context, errors::ErrorType},
  store::errors::{handle_db_error, DBError},
};
use serde_json::from_value;
use sqlx::Row;

use crate::store::database::dbstore::ProductsStoreImpl;

pub(super) async fn product_snapshot(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
  req: &ProductSnapshotRequest,
) -> Result<ProductSnapshot, DBError> {
  let path = "products.store.product_snapshot".to_string();

  let db = &*s.db.get().await;

  let row =
    sqlx::query("SELECT id, title, version, schema_version, offer FROM products WHERE id = $1")
      .bind(req.product_id.clone())
      .fetch_one(db)
      .await
      .map_err(|err| handle_db_error(err, "products.store.categories_init"))?;

  let offer: ProductOffer = from_value(row.get::<Option<serde_json::Value>, _>("offer").unwrap())
    .map_err(|err| {
    DBError::new(
      ErrorType::JsonUnmarshal,
      Box::new(err),
      "failed to deserialize product's offer",
      path,
      "",
    )
  })?;

  let product_snapshot = ProductSnapshot {
    id: row.get("id"),
    title: row.get("title"),
    version: row.get::<i16, _>("version") as u32, // Cast i16 to u32
    schema_version: row.get::<i16, _>("schema_version") as u32, // Cast i16 to u32
    offer: Some(offer),
  };

  Ok(product_snapshot)
}
