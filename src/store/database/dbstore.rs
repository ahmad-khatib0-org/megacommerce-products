mod product_create;
mod product_list;
mod router;

use std::sync::Arc;

use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct ProductsStoreImpl {
  pub(crate) db: Arc<Pool<Postgres>>,
}

#[derive(Debug)]
pub struct ProductsStoreImplArgs {
  pub db: Arc<Pool<Postgres>>,
}

impl ProductsStoreImpl {
  pub fn new(args: ProductsStoreImplArgs) -> Self {
    Self { db: args.db }
  }
}
