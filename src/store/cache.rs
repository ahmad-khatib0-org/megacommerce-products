mod categories;
mod tags;

use std::{
  collections::HashMap,
  error::Error,
  sync::{Arc, RwLock},
};

use megacommerce_proto::{ProductCategoriesWithoutSubcategories, ProductCategory, ProductTag};
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct Cache {
  db: Arc<Pool<Postgres>>,
  tags: RwLock<Vec<ProductTag>>,
  categories: RwLock<HashMap<String, ProductCategory>>,
  categories_without_subcategories: RwLock<ProductCategoriesWithoutSubcategories>,
}

#[derive(Debug)]
pub struct CacheArgs {
  pub db: Arc<Pool<Postgres>>,
}

impl Cache {
  pub async fn new(args: CacheArgs) -> Result<Self, Box<dyn Error + Send + Sync>> {
    let mut cache = Self {
      db: args.db,
      tags: RwLock::new(vec![]),
      categories: RwLock::new(HashMap::new()),
      categories_without_subcategories: RwLock::new(ProductCategoriesWithoutSubcategories {
        categories: vec![],
      }),
    };

    cache.tags_init().await?;
    cache.categories_init().await?;

    Ok(cache)
  }
}
