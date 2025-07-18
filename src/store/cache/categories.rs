use std::{collections::HashMap, error::Error};

use megacommerce_proto::{
  ProductCategoriesWithoutSubcategories, ProductCategory, ProductCategoryWithoutSubcategories,
  ProductSubcategory,
};
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
  pub fn categories(&self) -> ProductCategoriesWithoutSubcategories {
    self.categories_without_subcategories.read().unwrap().clone()
  }

  pub fn category_data(&self, category_name: &str) -> Option<ProductCategory> {
    println!("the category is: {}", &category_name);
    self.categories.read().unwrap().get(category_name).cloned()
  }

  pub(super) async fn categories_init(&self) -> Result<(), Box<dyn Error + Sync + Send>> {
    let rows = query!("SELECT id, name, subcategories FROM categories")
      .fetch_all(self.db.as_ref())
      .await
      .map_err(|err| handle_db_error(err, "products.store.categories_init"))?;

    let mut parsed_categories = HashMap::new();
    let mut parsed_cats_without_sub = Vec::new();

    for c in rows {
      let subcategories: Vec<ProductSubcategory> =
        from_value(c.subcategories).map_err(|err| DBError {
          err: Box::new(err),
          msg: "failed to process subcategories of a category".into(),
          path: "products.store.categories_init".into(),
          details: "".into(),
          err_type: ErrorType::JsonUnmarshal,
        })?;

      parsed_categories.insert(
        c.id.clone(),
        ProductCategory { id: c.id.clone(), name: c.name.clone(), subcategories },
      );

      parsed_cats_without_sub.push(ProductCategoryWithoutSubcategories { id: c.id, name: c.name });
    }

    *self.categories.write().unwrap() = parsed_categories;
    *self.categories_without_subcategories.write().unwrap() =
      ProductCategoriesWithoutSubcategories { categories: parsed_cats_without_sub };

    Ok(())
  }
}
