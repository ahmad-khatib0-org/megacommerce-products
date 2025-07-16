use std::{error::Error, sync::RwLockReadGuard};

use megacommerce_proto::{ProductCategory, ProductSubcategory};
use serde_json::from_value;
use sqlx::query;

use crate::{
  models::errors::ErrorType,
  store::{
    cache::Cache,
    database::errors::{handle_db_error, DBError},
  },
};

impl Cache {
  pub fn categories(&self) -> RwLockReadGuard<'_, Vec<ProductCategory>> {
    self.categories.read().unwrap()
  }

  pub(super) async fn categories_init(&self) -> Result<(), Box<dyn Error + Sync + Send>> {
    let rows = query!("SELECT id, name, subcategories FROM categories")
      .fetch_all(self.db.as_ref())
      .await
      .map_err(|err| handle_db_error(err, "products.store.categories_init"))?;

    let parsed_categories: Vec<ProductCategory> = rows
      .into_iter()
      .map(|c| {
        let subcategories: Vec<ProductSubcategory> =
          from_value(c.subcategories).map_err(|err| DBError {
            err: Box::new(err),
            msg: "failed to process subcategories".into(),
            path: "products.store.categories_init".into(),
            details: "".into(),
            err_type: ErrorType::JsonUnmarshal,
          })?;

        Ok(ProductCategory { id: c.id, name: c.name, subcategories })
      })
      .collect::<Result<Vec<ProductCategory>, DBError>>()?; // <-- this is the key

    let mut categories = self.categories.write().unwrap();
    *categories = parsed_categories;

    Ok(())
  }
}
