mod categories;
mod tags;

use std::{
  error::Error,
  sync::{Arc, RwLock},
  vec,
};

use megacommerce_proto::{ProductCategory, ProductTag};
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct Cache {
  tags: RwLock<Vec<ProductTag>>,
  categories: RwLock<Vec<ProductCategory>>,
  db: Arc<Pool<Postgres>>,
}

#[derive(Debug)]
pub struct CacheArgs {
  pub db: Arc<Pool<Postgres>>,
}

impl Cache {
  pub async fn new(args: CacheArgs) -> Result<Self, Box<dyn Error + Send + Sync>> {
    let mut cache =
      Self { tags: RwLock::new(vec![]), categories: RwLock::new(vec![]), db: args.db };

    cache.tags_init().await?;
    cache.categories_init().await?;

    Ok(cache)
  }
}
