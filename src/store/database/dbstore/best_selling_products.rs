use std::sync::Arc;

use megacommerce_proto::{
  BestSellingProductListItem, ProductMedia, ProductMediaImage, ProductOffer, ProductOfferVariant,
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

use crate::store::database::dbstore::ProductsStoreImpl;

// TODO: fetch the actual rating
pub(super) async fn best_selling_products(
  s: &ProductsStoreImpl,
  _: Arc<Context>,
) -> Result<Vec<BestSellingProductListItem>, DBError> {
  let path = "products.store.best_selling_products".to_string();
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

  // Use query_as! if you have a struct that matches the query result
  let rows = sqlx::query!(
    r#"
    SELECT
      ii.quantity_reserved,
      p.id,
      p.title,
      p.offer,
      p.media
    FROM
      inventory_items AS ii
    INNER JOIN products AS p ON ii.product_id = p.id
    ORDER BY ii.quantity_reserved DESC LIMIT 6
  "#
  )
  .fetch_all(db)
  .await
  .map_err(|err| de(Box::new(err), "failed to select inventory items", None))?;

  let mut best_sellers = Vec::new();

  for row in rows {
    let offer_data: ProductOffer = from_value(row.offer).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's offer", Some(ErrorType::JsonUnmarshal))
    })?;
    let media: ProductMedia = from_value(row.media).map_err(|err| {
      de(Box::new(err), "failed to deserialize product's media", Some(ErrorType::JsonUnmarshal))
    })?;

    let offer_vec: Vec<(String, ProductOfferVariant)> = offer_data.offer.into_iter().collect();
    let (variant_id, offer) = offer_vec.get(0).unwrap();

    let images_data: Vec<(String, ProductMediaImage)> =
      media.media.get(variant_id).unwrap().images.clone().into_iter().collect();
    let (_, image) = images_data.get(0).unwrap();

    let price =
      offer.price.parse::<f64>().expect(&format!("{}: Should convert price string to f64", path));
    best_sellers.push(BestSellingProductListItem {
      id: row.id,
      title: row.title,
      image: image.url.clone(),
      price_cents: (price * 100.0).round() as i64,
      sale_price_cents: offer
        .sale_price
        .as_ref()
        .and_then(|sp| sp.parse::<f64>().ok().and_then(|f| Some((f * 100.0).round() as i64))),
      rating: rand::rng().random_range(10..49),
      sold_count: row.quantity_reserved as u32,
    });
  }

  Ok(best_sellers)
}
