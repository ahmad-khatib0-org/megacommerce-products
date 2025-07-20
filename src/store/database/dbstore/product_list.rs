use std::sync::Arc;

use bigdecimal::BigDecimal;
use derive_more::Display;
use megacommerce_proto::{ProductListItem, ProductListRequest};
use sqlx::{postgres::PgRow, query, Error, Row};

use crate::{
  models::{
    context::Context,
    errors::{BoxedError, ErrorType},
  },
  store::database::{dbstore::ProductsStoreImpl, errors::DBError},
};

pub(super) async fn product_list(
  s: &ProductsStoreImpl,
  _ctx: Arc<Context>,
  request: &ProductListRequest,
) -> Result<Vec<ProductListItem>, DBError> {
  let mk_err = |msg: Option<&str>, err: BoxedError| DBError {
    err_type: ErrorType::Internal,
    err,
    msg: msg.unwrap_or("failed to query products").into(),
    path: "products.store.product_list".into(),
    details: "".into(),
  };

  let mut _where = "";
  if request.page > 1 {
    _where = " WHERE id < $1 ";
  }

  let sql = format!(
    r#"
      SELECT id, title, description, slug, price, currency_code, ar_enabled
      FROM products
      {}
      ORDER BY created_at DESC LIMIT 10
  "#,
    _where,
  );

  let result: Result<Vec<PgRow>, Error>;
  if request.page > 1 {
    result = query(&sql).bind(request.last_id.clone()).fetch_all(s.db.as_ref()).await;
  } else {
    result = query(&sql).fetch_all(s.db.as_ref()).await;
  };

  if let Err(err) = result {
    return Err(mk_err(None, Box::new(err)));
  }

  let products: Vec<ProductListItem> = result
    .unwrap()
    .into_iter()
    .map(|row| {
      let price = (row.try_get::<BigDecimal, _>("price").unwrap_or_default()).to_string();
      ProductListItem {
        id: row.try_get("id").unwrap_or_default(),
        user_id: "".to_string(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        slug: row.try_get("slug").unwrap_or_default(),
        price,
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        ar_enabled: row.try_get("ar_enabled").unwrap_or_default(),
      }
    })
    .collect();

  Ok(products)
}
