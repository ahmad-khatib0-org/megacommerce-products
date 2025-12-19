use sqlx::{FromRow, QueryBuilder};
use std::sync::Arc;

use megacommerce_proto::{ProductMedia, ProductOffer, ProductsCategoryItem};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::from_value;

use crate::store::database::dbstore::ProductsStoreImpl;

#[derive(FromRow)]
struct ProductCategoryRow {
  id: String,
  title: String,
  media: serde_json::Value,
  offer: serde_json::Value,
  created_at: i64,
  sold_count: i64,
}

pub(super) async fn products_category(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
  category_id: &str,
  subcategory_ids: &[String],
  page: u32,
  last_id: &str,
  limit: i64,
  sort_by: Option<&str>,
  sort_direction: Option<&str>,
) -> Result<Vec<ProductsCategoryItem>, DBError> {
  let path = "products.store.products_category".to_string();
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

  let order_clause = match (sort_by, sort_direction) {
    (Some("price"), Some("asc")) => "min_price ASC, id ASC",
    (Some("price"), Some("desc")) | (Some("price"), _) => "min_price DESC, id DESC",
    (Some("created_at"), Some("asc")) => "created_at ASC, id ASC",
    (Some("created_at"), Some("desc")) | (Some("created_at"), _) => "created_at DESC, id DESC",
    _ => "created_at DESC, id DESC",
  };

  // --- Start of the QueryBuilder implementation ---
  let mut query_builder = QueryBuilder::new(
    r#"
    WITH product_variants AS (
        SELECT
            p.id,
            p.title,
            p.media,
            p.offer,
            p.created_at,
            COALESCE(SUM(ii.quantity_reserved), 0)::BIGINT as sold_count,
            (SELECT MIN(
                CASE
                    WHEN (value->>'sale_price') IS NOT NULL AND (value->>'sale_price')::numeric > 0
                    THEN (value->>'sale_price')::numeric
                    ELSE (value->>'price')::numeric
                END
            ) FROM jsonb_each(p.offer->'offer'))::FLOAT8 as min_price
        FROM products AS p
        LEFT JOIN inventory_items AS ii ON ii.product_id = p.id
        WHERE
            p.category = 
    "#,
  );

  query_builder.push_bind(category_id);

  query_builder.push(" AND p.status = 'published'");

  if !subcategory_ids.is_empty() {
    query_builder.push(" AND p.subcategory = ANY(");
    query_builder.push_bind(subcategory_ids);
    query_builder.push("::TEXT[])");
  }

  if page > 1 {
    query_builder.push(" AND p.id < ");
    query_builder.push_bind(last_id);
  }

  query_builder.push(
    r#"
        GROUP BY p.id, p.title, p.media, p.offer, p.created_at, min_price
    )
    SELECT
        id,
        title,
        media,
        offer,
        created_at,
        sold_count,
        min_price
    FROM product_variants
    ORDER BY 
    "#,
  );

  query_builder.push(order_clause);

  query_builder.push(" LIMIT ");
  query_builder.push_bind(limit);

  let rows =
    query_builder.build_query_as::<ProductCategoryRow>().fetch_all(db).await.map_err(|err| {
      de(Box::new(err), "failed to fetch products from database", Some(ErrorType::DBSelectError))
    })?;

  // --- The rest of your post-processing logic remains the same ---
  let products: Vec<ProductsCategoryItem> = rows
    .into_iter()
    .filter_map(|row| {
      let offer_data: ProductOffer = from_value(row.offer)
        .map_err(|err| de(Box::new(err), "failed to deserialize offer", None))
        .ok()?;

      let media: ProductMedia = from_value(row.media)
        .map_err(|err| de(Box::new(err), "failed to deserialize media", None))
        .ok()?;

      let (variant_id, offer_variant) = offer_data.offer.into_iter().min_by(|(_, a), (_, b)| {
        let price_a =
          if a.has_sale_price { a.sale_price.as_deref().unwrap_or(&a.price) } else { &a.price };

        let price_b =
          if b.has_sale_price { b.sale_price.as_deref().unwrap_or(&b.price) } else { &b.price };

        price_a.partial_cmp(price_b).unwrap_or(std::cmp::Ordering::Equal)
      })?;

      let image_url = media
        .media
        .get(&variant_id)
        .and_then(|vm| vm.images.values().next())
        .map(|img| img.url.clone())
        .unwrap_or_default();

      let price_str = if offer_variant.has_sale_price {
        offer_variant.sale_price.as_deref().unwrap_or(&offer_variant.price)
      } else {
        &offer_variant.price
      };

      let price = price_str.parse::<f64>().ok()?;
      let price_cents = (price * 100.0).round() as u32;

      let (discount_price_cents, discount_percentage) =
        if let Some(sale_price_str) = &offer_variant.sale_price {
          if let (Ok(original_price), Ok(sale_price)) =
            (offer_variant.price.parse::<f64>(), sale_price_str.parse::<f64>())
          {
            let disc_cents = (sale_price * 100.0).round() as u32;
            let disc_pct = ((original_price - sale_price) / original_price * 100.0).round() as u32;
            (Some(disc_cents), Some(disc_pct))
          } else {
            (None, None)
          }
        } else {
          (None, None)
        };

      Some(ProductsCategoryItem {
        id: row.id,
        variant_id: variant_id.to_string(),
        title: row.title,
        image: image_url,
        price_cents,
        discount_price_cents,
        discount_percentage,
        sold_by: "Megacommerce".to_string(),
        rating: None,
        sold_count: Some(row.sold_count as u32),
        created_at: row.created_at as u64,
      })
    })
    .collect();

  Ok(products)
}
