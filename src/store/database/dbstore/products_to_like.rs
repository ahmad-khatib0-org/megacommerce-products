use std::io::{Error, ErrorKind};

use bigdecimal::ToPrimitive;
use megacommerce_proto::{
  ProductMedia, ProductOffer, ProductOfferVariant, ProductPrice, ProductToLikeListItem,
};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use rand::Rng;
use serde_json::from_value;
use sqlx::FromRow;
use std::sync::Arc;

use crate::store::database::dbstore::ProductsStoreImpl;

// Helper struct for type-safe database row mapping
#[derive(FromRow)]
struct ProductRow {
  id: String,
  title: String,
  media: serde_json::Value,
  offer: serde_json::Value,
  sold_count: i64,
}

pub(super) async fn products_to_like(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
  page: u32,
  last_id: &str,
  limit: i64,
) -> Result<Vec<ProductToLikeListItem>, DBError> {
  let path = "products.store.products_to_like";
  let de = |err: BoxedErr, msg: &str, err_type: Option<ErrorType>| -> DBError {
    DBError::new(
      err_type.unwrap_or(ErrorType::DBSelectError),
      err,
      msg.to_string(),
      path,
      "".to_string(),
    )
  };

  let db = &*s.db.get().await;

  let where_clause = if page > 1 { " WHERE p.id < $1 " } else { "" };
  let limit_clause = if page > 1 { " LIMIT $2 " } else { " LIMIT $1 " };

  // Use a more robust query with aggregation to avoid duplicate products
  let sql = format!(
    r#"
        SELECT
            p.id,
            p.title,
            p.media,
            p.offer,
            COALESCE(SUM(ii.quantity_reserved), 0)::BIGINT as sold_count
        FROM products AS p
        LEFT JOIN inventory_items AS ii ON ii.product_id = p.id
        {}
        GROUP BY p.id, p.title, p.media, p.offer
        ORDER BY p.id DESC
        {}
        "#,
    where_clause, limit_clause,
  );

  let query_builder = sqlx::query_as::<_, ProductRow>(&sql);

  let rows =
    if page > 1 { query_builder.bind(last_id).bind(limit) } else { query_builder.bind(limit) }
      .fetch_all(db)
      .await
      .map_err(|err| {
        de(Box::new(err), "failed to fetch products from database", Some(ErrorType::DBSelectError))
      })?;

  let products: Vec<ProductToLikeListItem> = rows
    .into_iter()
    .map(|row| {
      let offer_data: ProductOffer = from_value(row.offer)
        .map_err(|err| de(Box::new(err), "failed to deserialize product's offer", None))?;
      let media: ProductMedia = from_value(row.media)
        .map_err(|err| de(Box::new(err), "failed to deserialize product's media", None))?;

      let offer_vec: Vec<(String, ProductOfferVariant)> = offer_data.offer.into_iter().collect();
      let (variant_id, offer_variant) = offer_vec.into_iter().next().ok_or_else(|| {
        de(
          Box::new(Error::new(ErrorKind::NotFound, "Product has no variants")),
          "No variant found",
          Some(ErrorType::NotFound),
        )
      })?;

      let image_url = media
        .media
        .get(&variant_id)
        .and_then(|variant_media| variant_media.images.values().next())
        .map(|img| img.url.clone())
        .unwrap_or_default();

      let price = offer_variant
        .price
        .parse::<f64>()
        .map_err(|err| de(Box::new(err), "failed to parse price", None))?;
      let price_cents = (price * 100.0).round() as u32;
      let formatted_price = format!("${:.2}", price);

      let mut product_price =
        ProductPrice { amount: price, formatted: formatted_price, ..Default::default() };

      if let Some(sale_price_str) = &offer_variant.sale_price {
        let sale_price = sale_price_str.parse::<f64>().map_err(|err| {
          de(Box::new(err), "failed to parse sale price", Some(ErrorType::InvalidNumber))
        })?;
        let discount_price_cents = (sale_price * 100.0).round() as u32;
        let save_amount_cents = price_cents - discount_price_cents;
        let save_percentage = ((price - sale_price) / price * 100.0).round() as u32;

        product_price.discount_price = Some(sale_price);
        product_price.save_amount = Some(format!("${:.2}", save_amount_cents as f64 / 100.0));
        product_price.save_percentage = Some(format!("{}%", save_percentage));
      }

      Ok(ProductToLikeListItem {
        id: row.id,
        variant_id: variant_id.to_string(),
        title: row.title,
        image: image_url,
        price: Some(product_price),
        rating: (rand::rng().random_range(10..49) / 10).to_f64(),
        sold: Some(row.sold_count as i32),
        meta: vec![],
      })
    })
    .collect::<Result<Vec<ProductToLikeListItem>, DBError>>()?;

  Ok(products)
}
