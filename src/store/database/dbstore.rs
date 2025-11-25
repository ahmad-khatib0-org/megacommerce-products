mod best_selling_products;
mod product_create;
mod product_list;
mod product_snapshot;
mod router;

use megacommerce_shared::models::r_lock::RLock;
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct ProductsStoreImpl {
  pub(crate) db: RLock<Pool<Postgres>>,
}

#[derive(Debug)]
pub struct ProductsStoreImplArgs {
  pub db: RLock<Pool<Postgres>>,
}

impl ProductsStoreImpl {
  pub fn new(args: ProductsStoreImplArgs) -> Self {
    Self { db: args.db }
  }
}
