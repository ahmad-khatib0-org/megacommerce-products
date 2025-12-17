use std::sync::Arc;

use megacommerce_proto::{ProductBulletPoint, ProductDetailsResponseData, ProductTag};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::from_value;

use crate::store::database::dbstore::ProductsStoreImpl;

pub(super) async fn product_details(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
  id: &str,
) -> Result<ProductDetailsResponseData, DBError> {
  let path = "products.store.product_details";
  let de = |err: BoxedErr, err_type: ErrorType, msg: &str| {
    DBError::new(err_type, err, msg, path, "".to_string())
  };

  let db = &*s.db.get().await;

  let row = sqlx::query!(
    r#"
    SELECT 
      id, user_id, title, category, subcategory, has_variations,
      brand_name, product_id, product_id_type, description,
      bullet_points, currency_code, fulfillment_type, processing_time,
      details, media, offer, safety, tags, metadata
    FROM products 
    WHERE id = $1
    "#,
    id
  )
  .fetch_one(db)
  .await
  .map_err(|err| de(Box::new(err), ErrorType::DBSelectError, "failed to select product details"))?;

  let bullet_points: Vec<ProductBulletPoint> = from_value(row.bullet_points).map_err(|err| {
    de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize bullet_points")
  })?;

  let details: megacommerce_proto::ProductDetails = from_value(row.details)
    .map_err(|err| de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize details"))?;

  let media: megacommerce_proto::ProductMedia = from_value(row.media)
    .map_err(|err| de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize media"))?;

  let offer: megacommerce_proto::ProductOffer = from_value(row.offer)
    .map_err(|err| de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize offer"))?;

  let safety: megacommerce_proto::ProductSafety = from_value(row.safety)
    .map_err(|err| de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize safety"))?;

  let tags: Vec<ProductTag> = from_value(row.tags).map_err(|err| {
    de(
      Box::new(err),
      ErrorType::JsonUnmarshal,
      &format!("failed to deserialize tags, product id:  {}", row.id),
    )
  })?;

  let metadata: Option<megacommerce_proto::ProductMetadata> = match row.metadata {
    Some(value) => Some(from_value(value).map_err(|err| {
      de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize metadata")
    })?),
    None => None,
  };

  // Construct response
  Ok(ProductDetailsResponseData {
    id: row.id,
    supplier_id: row.user_id,
    title: row.title,
    category: row.category,
    subcategory: row.subcategory,
    has_variations: row.has_variations,
    brand_name: row.brand_name,
    product_id: row.product_id,
    product_id_type: row.product_id_type,
    description: row.description,
    bullet_points,
    currency_code: row.currency_code,
    fulfillment_type: row.fulfillment_type,
    processing_time: row.processing_time as u64,
    details: Some(details),
    media: Some(media),
    offer: Some(offer),
    safety: Some(safety),
    tags,
    metadata,
  })
}
