pub mod dbstore;

use megacommerce_proto::{
  BestSellingProductListItem, BigDiscountProductListItem, HeroProductsResponseData,
  NewlyAddedProductListItem, Product, ProductSnapshot, ProductSnapshotRequest,
  ProductToLikeListItem,
};
use megacommerce_shared::{models::context::Context, store::errors::DBError};
use std::{fmt, sync::Arc};

#[tonic::async_trait]
pub trait ProductsStore: fmt::Debug + Send + Sync {
  async fn product_create(&self, ctx: Arc<Context>, product: &Product) -> Result<(), DBError>;
  async fn products_to_like(
    &self,
    ctx: Arc<Context>,
    page: u32,
    last_id: &str,
    limit: i64,
  ) -> Result<Vec<ProductToLikeListItem>, DBError>;
  async fn product_snapshot(
    &self,
    ctx: Arc<Context>,
    request: &ProductSnapshotRequest,
  ) -> Result<ProductSnapshot, DBError>;
  async fn best_selling_products(
    &self,
    _: Arc<Context>,
  ) -> Result<Vec<BestSellingProductListItem>, DBError>;
  async fn big_discount_products(
    &self,
    _: Arc<Context>,
  ) -> Result<Vec<BigDiscountProductListItem>, DBError>;
  async fn newly_added_products(
    &self,
    _: Arc<Context>,
  ) -> Result<Vec<NewlyAddedProductListItem>, DBError>;
  async fn hero_products(&self, ctx: Arc<Context>) -> Result<HeroProductsResponseData, DBError>;
}
