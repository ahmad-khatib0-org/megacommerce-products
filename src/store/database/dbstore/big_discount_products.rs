use std::sync::Arc;

use bigdecimal::num_traits::ToPrimitive;
use megacommerce_proto::{BigDiscountProductListItem, ProductMedia};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::from_value;

use crate::store::database::dbstore::ProductsStoreImpl;

pub(super) async fn big_discount_products(
  s: &ProductsStoreImpl,
  _: Arc<Context>,
) -> Result<Vec<BigDiscountProductListItem>, DBError> {
  let path = "products.store.big_discount_products".to_string();
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
          variant.key as variant_id,
          (variant.value ->> 'price')::numeric as price,
          (variant.value ->> 'sale_price')::numeric as sale_price,
          ((variant.value ->> 'price')::numeric - (variant.value ->> 'sale_price')::numeric) / (variant.value ->> 'price')::numeric as discount_percentage
        FROM products p, 
        jsonb_each(p.offer -> 'offer') as variant
        WHERE 
              (variant.value ->> 'has_sale_price')::boolean = true 
              AND (
                (variant.value ->> 'sale_price_end') IS NULL 
                OR (variant.value ->> 'sale_price_end')::bigint > EXTRACT(EPOCH FROM NOW()) * 1000
              )
      )
      SELECT
          v.id,
          v.title,
          v.media,
          v.variant_id,
          v.price,
          v.sale_price,
          v.discount_percentage,
          COALESCE(ii.quantity_reserved, 0) AS sold_count
      FROM variants AS v
      LEFT JOIN inventory_items AS ii ON ii.variant_id = v.variant_id
      ORDER BY v.discount_percentage DESC
      LIMIT 6
    "#
  )
  .fetch_all(db)
  .await
  .map_err(|err| de(Box::new(err), "failed to select big discount products", None))?;

  let mut big_discount_products = Vec::new();

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

    let discount_percentage =
      (row.discount_percentage.unwrap_or_default().to_f64().unwrap_or(0.0) * 100.0).round() as u32;

    big_discount_products.push(BigDiscountProductListItem {
      id: row.id.unwrap_or_default(),
      variant_id: variant_id.to_string(),
      title: row.title.unwrap_or_default(),
      image: image_url,
      price_cents,
      discount_price_cents,
      discount_percentage,
      sold_count: row.sold_count.unwrap_or_default() as u32,
    });
  }

  Ok(big_discount_products)
}
