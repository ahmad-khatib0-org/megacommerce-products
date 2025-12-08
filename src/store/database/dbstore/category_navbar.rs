use std::io::Error;
use std::{io::ErrorKind, sync::Arc};

use megacommerce_proto::{
  CategoryNavbarProductItem, CategoryNavbarResponseData, ProductMedia, ProductOffer,
  ProductOfferVariant,
};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::from_value;

use crate::store::database::dbstore::ProductsStoreImpl;

pub(super) async fn category_navbar(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
  category_id: &str,
  subcategory_id: &str,
) -> Result<CategoryNavbarResponseData, DBError> {
  let path = "products.store.category_navbar".to_string();
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

  // Fetch category and subcategory info from categories table
  let category_row = sqlx::query!(
    r#"
    SELECT 
      name,
      subcategories
    FROM 
      categories
    WHERE 
      id = $1
    "#,
    category_id
  )
  .fetch_optional(db)
  .await
  .map_err(|err| de(Box::new(err), "failed to select from categories table", None))?
  .ok_or_else(|| {
    de(
      Box::new(Error::new(ErrorKind::NotFound, format!("Category {} not found", category_id))),
      "category not found",
      Some(ErrorType::NoRows),
    )
  })?;

  // Parse subcategories JSON to find matching subcategory name
  let subcategories: Vec<serde_json::Value> = serde_json::from_value(category_row.subcategories)
    .map_err(|err| {
      de(Box::new(err), "failed to parse subcategories", Some(ErrorType::JsonUnmarshal))
    })?;

  let subcategory_name = subcategories
    .iter()
    .find_map(|sc| {
      if sc.get("id").and_then(|id| id.as_str()) == Some(subcategory_id) {
        sc.get("name").and_then(|name| name.as_str()).map(|s| s.to_string())
      } else {
        None
      }
    })
    .ok_or_else(|| {
      de(
        Box::new(Error::new(
          ErrorKind::NotFound,
          format!("Subcategory {} not found in category {}", subcategory_id, category_id),
        )),
        "subcategory not found",
        Some(ErrorType::NoRows),
      )
    })?;

  // Fetch random products from the category/subcategory (max 6)
  // For now, using random ordering since ML integration is deferred
  let product_rows = sqlx::query!(
    r#"
    SELECT 
      p.id,
      p.title,
      p.media,
      p.offer
    FROM 
      products AS p
    WHERE 
      p.category = $1 
      AND p.subcategory = $2 
      AND p.status = 'published'
    ORDER BY 
      RANDOM()
    LIMIT 6
    "#,
    category_id,
    subcategory_id
  )
  .fetch_all(db)
  .await
  .map_err(|err| de(Box::new(err), "failed to select products by category/subcategory", None))?;

  let mut recommended_products = Vec::new();

  for row in product_rows {
    let offer_data: ProductOffer = from_value(row.offer).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's offer", Some(ErrorType::JsonUnmarshal))
    })?;
    let media: ProductMedia = from_value(row.media).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's media", Some(ErrorType::JsonUnmarshal))
    })?;

    // Get first variant (or random variant if multiple)
    let offer_vec: Vec<(String, ProductOfferVariant)> = offer_data.offer.into_iter().collect();
    let (variant_id, offer) = offer_vec.get(0).unwrap();

    let images_data: Vec<(String, _)> =
      media.media.get(variant_id).unwrap().images.clone().into_iter().collect();
    let (_, image) = images_data.get(0).unwrap();

    let price =
      offer.price.parse::<f64>().expect(&format!("{}: Should convert price string to f64", path));
    let sale_price = offer.sale_price.as_ref().and_then(|sp| sp.parse::<f64>().ok()).unwrap_or(0.0);

    let price_cents = (price * 100.0).round() as u32;
    let discount_price_cents = (sale_price * 100.0).round() as u32;

    let discount_percentage =
      if sale_price > 0.0 { ((price - sale_price) / price * 100.0).round() as u32 } else { 0 };

    // TODO: fetch actual seller name from users table
    recommended_products.push(CategoryNavbarProductItem {
      id: row.id,
      variant_id: variant_id.to_string(),
      title: row.title,
      image: image.url.clone(),
      price_cents,
      discount_price_cents: if sale_price > 0.0 { Some(discount_price_cents) } else { None },
      discount_percentage: if sale_price > 0.0 { Some(discount_percentage) } else { None },
      sold_by: "Megacommerce".to_string(),
    });
  }

  Ok(CategoryNavbarResponseData {
    category_id: category_id.to_string(),
    category_name: category_row.name,
    subcategory_id: subcategory_id.to_string(),
    subcategory_name,
    recommended_products,
  })
}
