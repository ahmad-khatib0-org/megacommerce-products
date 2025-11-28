use std::sync::Arc;

use megacommerce_proto::{
  BestSellingProductListItem, BigDiscountProductListItem, NewlyAddedProductListItem, Product,
  ProductListItem, ProductListRequest, ProductSnapshot, ProductSnapshotRequest,
};
use megacommerce_shared::{models::context::Context, store::errors::DBError};

use crate::store::database::{
  dbstore::{
    best_selling_products::best_selling_products, big_discount_products::big_discount_products,
    newly_added_products::newly_added_products, product_create::product_create,
    product_list::product_list, product_snapshot::product_snapshot, ProductsStoreImpl,
  },
  ProductsStore,
};

#[tonic::async_trait]
impl ProductsStore for ProductsStoreImpl {
  async fn product_create(&self, ctx: Arc<Context>, product: &Product) -> Result<(), DBError> {
    product_create(self, ctx, product).await
  }
  async fn product_list(
    &self,
    ctx: Arc<Context>,
    product: &ProductListRequest,
  ) -> Result<Vec<ProductListItem>, DBError> {
    product_list(self, ctx, product).await
  }
  async fn product_snapshot(
    &self,
    ctx: Arc<Context>,
    req: &ProductSnapshotRequest,
  ) -> Result<ProductSnapshot, DBError> {
    product_snapshot(self, ctx, req).await
  }
  async fn best_selling_products(
    &self,
    ctx: Arc<Context>,
  ) -> Result<Vec<BestSellingProductListItem>, DBError> {
    best_selling_products(self, ctx).await
  }
  async fn big_discount_products(
    &self,
    ctx: Arc<Context>,
  ) -> Result<Vec<BigDiscountProductListItem>, DBError> {
    big_discount_products(self, ctx).await
  }
  async fn newly_added_products(
    &self,
    ctx: Arc<Context>,
  ) -> Result<Vec<NewlyAddedProductListItem>, DBError> {
    newly_added_products(self, ctx).await
  }
}
