use std::sync::Arc;
use std::{collections::HashMap, io::ErrorKind};

use megacommerce_proto::{
  HeroProductData, HeroProductListItem, HeroProductsResponseCategorySlider,
  HeroProductsResponseData, HeroProductsResponseWelcomeDealsSlider, ProductMedia, ProductOffer,
};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{BoxedErr, ErrorType},
  },
  store::errors::DBError,
};
use serde_json::{from_value, Value};
use sqlx::FromRow;

use crate::store::database::dbstore::ProductsStoreImpl;

#[derive(FromRow)]
struct ProductRow {
  id: String,
  title: String,
  media: Value,
  offer: Value,
}

struct ProductAndVariantID<'a> {
  product_id: &'a str,
  variant_id: &'a str,
}

// Helper function to process rows and convert them to HeroProductListItem
// This eliminates code duplication.
fn process_slider_rows(
  rows: Vec<ProductRow>,
  slider_ids: &[ProductAndVariantID],
  de: impl Fn(BoxedErr, ErrorType, &str) -> DBError,
) -> Result<Vec<HeroProductListItem>, DBError> {
  // using  a HashMap for efficient O(1) lookups of variant_id by product_id
  let variant_map: HashMap<&str, &str> =
    slider_ids.iter().map(|item| (item.product_id, item.variant_id)).collect();

  let mut products = Vec::with_capacity(rows.len());

  for pro in rows {
    let variant_id = *variant_map.get(&pro.id as &str).ok_or_else(|| {
      de(
        Box::new(std::io::Error::new(
          ErrorKind::NotFound,
          format!("Variant ID for product {} not found in slider config", pro.id),
        )),
        ErrorType::NoRows,
        "failed to find variant for product",
      )
    })?;

    let offer: ProductOffer = from_value(pro.offer).map_err(|err| {
      de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize product's offer")
    })?;

    let offer_variant = offer.offer.get(variant_id).ok_or_else(|| {
      de(
        Box::new(std::io::Error::new(
          ErrorKind::NotFound,
          format!("Offer variant {} not found for product {}", variant_id, pro.id),
        )),
        ErrorType::NoRows,
        "failed to find offer variant",
      )
    })?;

    let media: ProductMedia = from_value(pro.media).map_err(|err| {
      de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize product's media")
    })?;

    let image = media
      .media
      .get(variant_id)
      .and_then(|variant_media| variant_media.images.values().next())
      .map(|img| img.url.clone())
      .unwrap_or_default();

    let price = offer_variant.price.parse::<f64>().unwrap_or_default();
    let sale_price = offer_variant
      .sale_price
      .as_ref()
      .map(|s| s.parse::<f64>().unwrap_or_default())
      .unwrap_or(0.0);

    let price_cents = (price * 100.0).round() as u32;
    let discount_price_cents = (sale_price * 100.0).round() as u32;

    // Corrected discount calculation
    let discount_percentage =
      if sale_price > 0.0 { ((price - sale_price) / price * 100.0).round() as u32 } else { 0 };

    products.push(HeroProductListItem {
      id: pro.id,
      variant_id: variant_id.to_string(),
      title: pro.title,
      image,
      price_cents,
      discount_price_cents: Some(discount_price_cents),
      discount_percentage: Some(discount_percentage),
    });
  }

  Ok(products)
}

pub(super) async fn hero_products(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
) -> Result<HeroProductsResponseData, DBError> {
  let path = "products.store.hero_products";
  let de = |err: BoxedErr, err_type: ErrorType, msg: &str| {
    DBError::new(err_type, err, msg, path, "".to_string())
  };

  let db = &*s.db.get().await;

  let row =
    sqlx::query!("SELECT id, products_data FROM hero_products ORDER BY created_at DESC LIMIT 1")
      .fetch_one(db)
      .await
      .map_err(|err| {
        de(Box::new(err), ErrorType::DBSelectError, "failed to select from hero_products table")
      })?;

  let products_data: HeroProductData = from_value(row.products_data).map_err(|err| {
    de(Box::new(err), ErrorType::JsonUnmarshal, "failed to deserialize products_data column")
  })?;

  let welcome = products_data.welcome_deals_slider.unwrap();
  let category = products_data.category_slider.unwrap();

  let welcome_slider_ids: Vec<ProductAndVariantID> = welcome
    .products
    .iter()
    .map(|pro| ProductAndVariantID { product_id: &pro.id, variant_id: &pro.variant_id })
    .collect();
  let category_slider_ids: Vec<ProductAndVariantID> = category
    .products
    .iter()
    .map(|pro| ProductAndVariantID { product_id: &pro.id, variant_id: &pro.variant_id })
    .collect();

  // Fetch product data for both sliders
  let welcome_rows: Vec<ProductRow> =
    sqlx::query_as("SELECT id, title, media, offer FROM products WHERE id = ANY($1::TEXT[])")
      .bind(&welcome_slider_ids.iter().map(|row| row.product_id).collect::<Vec<&str>>())
      .fetch_all(db)
      .await
      .map_err(|err| {
        de(Box::new(err), ErrorType::DBSelectError, "failed to select welcome_slider products")
      })?;

  let category_rows: Vec<ProductRow> =
    sqlx::query_as("SELECT id, title, media, offer FROM products WHERE id = ANY($1::TEXT[])")
      .bind(&category_slider_ids.iter().map(|row| row.product_id).collect::<Vec<&str>>())
      .fetch_all(db)
      .await
      .map_err(|err| {
        de(Box::new(err), ErrorType::DBSelectError, "failed to select category_slider products")
      })?;

  // Process rows using the helper function
  let welcome_products = process_slider_rows(welcome_rows, &welcome_slider_ids, &de)?;
  let category_products = process_slider_rows(category_rows, &category_slider_ids, &de)?;

  // Construct the final response
  let result = HeroProductsResponseData {
    welcome_deals_slider: Some(HeroProductsResponseWelcomeDealsSlider {
      title: welcome.title,
      subtitle: welcome.subtitle,
      button_text: welcome.button_text,
      products: welcome_products,
    }),
    category_slider: Some(HeroProductsResponseCategorySlider {
      title: category.title,
      subtitle: category.subtitle,
      button_text: category.button_text,
      products: category_products,
    }),
  };

  Ok(result)
}

