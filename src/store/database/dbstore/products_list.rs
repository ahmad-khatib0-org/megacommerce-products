use std::sync::Arc;

use megacommerce_proto::{ProductListItem, ProductMedia, ProductOffer};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::from_value;
use sqlx::Row;

use crate::store::database::dbstore::ProductsStoreImpl;

pub(super) async fn products_list(
  s: &ProductsStoreImpl,
  ctx: Arc<Context>,
  page: u32,
  last_id: &str,
  limit: i64,
) -> Result<Vec<ProductListItem>, DBError> {
  let path = "products.store.products_list";
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
  let user_id = ctx.session().user_id.clone();

  let where_clause = if page > 1 { " AND p.id < $2 " } else { "" };

  let sql = format!(
    r#"
        SELECT
            p.id,
            p.title,
            p.status,
            p.created_at,
            p.offer,
            p.media,
            p.currency_code
        FROM products AS p
        WHERE p.user_id = $1 {}
        ORDER BY p.id DESC
        LIMIT ${}
        "#,
    where_clause,
    if page > 1 { "3" } else { "2" }
  );

  let rows = if page > 1 {
    sqlx::query(&sql).bind(user_id).bind(last_id).bind(limit).fetch_all(db).await
  } else {
    sqlx::query(&sql).bind(user_id).bind(limit).fetch_all(db).await
  }
  .map_err(|err| de(Box::new(err), "failed to fetch products from database", None))?;

  let mut products = Vec::new();

  for row in rows {
    let offer_data: ProductOffer = from_value(row.get("offer")).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's offer", Some(ErrorType::JsonUnmarshal))
    })?;
    let media: ProductMedia = from_value(row.get("media")).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's media", Some(ErrorType::JsonUnmarshal))
    })?;

    // Get first variant
    let offer_vec: Vec<(String, _)> = offer_data.offer.into_iter().collect();
    let (variant_id, offer_variant) = match offer_vec.get(0) {
      Some((vid, ov)) => (vid.clone(), ov),
      None => {
        continue;
      }
    };

    // Get first image for this variant
    let image_url = if let Some(variant_media) = media.media.get(&variant_id) {
      if let Some((_, image)) = variant_media.images.iter().next() {
        image.url.clone()
      } else {
        "".to_string()
      }
    } else {
      "".to_string()
    };

    let price = offer_variant.price.parse::<f64>().unwrap_or(0.0);

    let list_price = offer_variant.list_price.as_ref().and_then(|lp| lp.parse::<f64>().ok());

    let sale_price = offer_variant.sale_price.as_ref().and_then(|sp| sp.parse::<f64>().ok());

    products.push(ProductListItem {
      id: row.get("id"),
      title: row.get("title"),
      status: row.get("status"),
      created_at: row.get::<i64, _>("created_at") as u64,
      price,
      list_price,
      sale_price,
      currency_code: row.get("currency_code"),
      quantity: offer_variant.quantity as i32,
      image: image_url,
    });
  }

  Ok(products)
}
