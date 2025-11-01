mod categories;
mod tags;

use std::sync::Arc;

use dashmap::DashMap;
use megacommerce_proto::{Category, ProductTag, Subcategory, SubcategoryTranslations};
use megacommerce_shared::models::errors::BoxedErr;
use parking_lot::RwLock;
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct Cache {
  db: Arc<Pool<Postgres>>,
  tags: RwLock<Vec<ProductTag>>,
  categories: DashMap<String, Arc<Category>>,
  /// category_id -> ( subcategory_id -> Arc<Subcategory> )
  subcategories_data: DashMap<String, DashMap<String, Arc<Subcategory>>>,
  /// (category_id, (subcategory_id , (language, SubcategoryTranslations)))
  subcategories_translation:
    DashMap<String, DashMap<String, DashMap<String, Arc<SubcategoryTranslations>>>>,
}

#[derive(Debug)]
pub struct CacheArgs {
  pub db: Arc<Pool<Postgres>>,
}

impl Cache {
  pub async fn new(args: CacheArgs) -> Result<Self, BoxedErr> {
    let mut cache = Self {
      db: args.db,
      tags: RwLock::new(vec![]),
      categories: DashMap::new(),
      subcategories_data: DashMap::new(),
      subcategories_translation: DashMap::new(),
    };

    cache.tags_init().await?;
    cache.categories_init().await?;

    Ok(cache)
  }
}
