mod best_selling_products;
mod big_discount_products;
mod category_navbar;
mod hero_products;
mod newly_added_products;
mod product_create;
mod product_details;
mod product_snapshot;
mod products_category;
mod products_list;
mod products_to_like;
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
