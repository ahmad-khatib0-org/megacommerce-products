use std::sync::Arc;

use bigdecimal::num_traits::ToPrimitive;
use megacommerce_proto::{NewlyAddedProductListItem, ProductMedia};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::from_value;

use crate::{
  models::time::format_human_readable_time, store::database::dbstore::ProductsStoreImpl,
};

pub(super) async fn newly_added_products(
  s: &ProductsStoreImpl,
  ctx: Arc<Context>,
) -> Result<Vec<NewlyAddedProductListItem>, DBError> {
  let path = "products.store.newly_added_products".to_string();
  let de = |err: BoxedErr, msg: &str, err_type: Option<ErrorType>| {
    DBError::new(
      err_type.unwrap_or(ErrorType::DBSelectError),
      err,
      msg.to_string(),
      &path,
      "".to_string(),
    )
  };

  let db = &*s.db.get().await;

  let rows = sqlx::query!(
    r#"
      WITH variants AS (
        SELECT
          p.id,
          p.title,
          p.media,
          p.created_at,
          variant.key as variant_id,
          (variant.value ->> 'price')::numeric as price,
          (variant.value ->> 'sale_price')::numeric as sale_price,
          ROW_NUMBER() OVER (
            PARTITION BY p.id ORDER BY (variant.value ->> 'price')::numeric DESC
          ) as rn
        FROM products p, 
        jsonb_each(p.offer -> 'offer') as variant
      )
      SELECT
          v.id,
          v.title,
          v.media,
          v.variant_id,
          v.price,
          v.sale_price,
          v.created_at
      FROM variants AS v
      WHERE v.rn = 1
      ORDER BY v.created_at DESC
      LIMIT 6
    "#
  )
  .fetch_all(db)
  .await
  .map_err(|err| de(Box::new(err), "failed to select newly added products", None))?;

  let mut newly_added_products = Vec::new();

  for row in rows {
    let media: ProductMedia = from_value(row.media.unwrap_or_default()).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's media", Some(ErrorType::JsonUnmarshal))
    })?;

    let variant_id = row.variant_id.unwrap_or_default();

    // Get the first image for this variant
    let image_url = if let Some(variant_media) = media.media.get(&variant_id) {
      if let Some((_, first_image)) = variant_media.images.iter().next() {
        first_image.url.clone()
      } else {
        "".to_string()
      }
    } else {
      "".to_string()
    };

    let price = row.price.unwrap_or_default().to_f64().unwrap_or(0.0);
    let sale_price = row.sale_price.unwrap_or_default().to_f64().unwrap_or(0.0);
    let price_cents = (price * 100.0).round() as u32;
    let discount_price_cents = (sale_price * 100.0).round() as u32;

    // Calculate discount percentage if sale price exists
    let discount_percentage =
      if sale_price > 0.0 { ((price - sale_price) / price * 100.0).round() as u32 } else { 0 };

    // Format the created_at timestamp
    let created_at_ms = row.created_at.unwrap_or_default() as i64;
    let timezone = &ctx.timezone;
    let created_at = format_human_readable_time(&ctx.accept_language, created_at_ms, timezone);

    newly_added_products.push(NewlyAddedProductListItem {
      id: row.id.unwrap_or_default(),
      title: row.title.unwrap_or_default(),
      image: image_url,
      price_cents,
      sale_price_cents: Some(discount_price_cents),
      discount_percentage: Some(discount_percentage),
      created_at,
    });
  }

  Ok(newly_added_products)
}
