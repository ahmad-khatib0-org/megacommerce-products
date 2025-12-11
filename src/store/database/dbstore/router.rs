use std::sync::Arc;

use megacommerce_proto::{
  BestSellingProductListItem, BigDiscountProductListItem, CategoryNavbarResponseData,
  HeroProductsResponseData, NewlyAddedProductListItem, Product, ProductDetailsResponseData,
  ProductsCategoryItem, ProductSnapshot, ProductSnapshotRequest, ProductToLikeListItem,
};
use megacommerce_shared::{models::context::Context, store::errors::DBError};

use crate::store::database::{
  dbstore::{
    best_selling_products::best_selling_products, big_discount_products::big_discount_products,
    category_navbar::category_navbar, hero_products::hero_products,
    newly_added_products::newly_added_products, product_create::product_create,
    product_details::product_details, product_snapshot::product_snapshot,
    products_category::products_category, products_to_like::products_to_like, ProductsStoreImpl,
  },
  ProductsStore,
};

#[tonic::async_trait]
impl ProductsStore for ProductsStoreImpl {
  async fn product_create(&self, ctx: Arc<Context>, product: &Product) -> Result<(), DBError> {
    product_create(self, ctx, product).await
  }
  async fn products_to_like(
    &self,
    ctx: Arc<Context>,
    page: u32,
    last_id: &str,
    limit: i64,
  ) -> Result<Vec<ProductToLikeListItem>, DBError> {
    products_to_like(self, ctx, page, last_id, limit).await
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
  async fn hero_products(&self, ctx: Arc<Context>) -> Result<HeroProductsResponseData, DBError> {
    hero_products(self, ctx).await
  }
  async fn product_details(
    &self,
    ctx: Arc<Context>,
    id: &str,
  ) -> Result<ProductDetailsResponseData, DBError> {
    product_details(self, ctx, id).await
  }
  async fn category_navbar(
    &self,
    ctx: Arc<Context>,
    category_id: &str,
    subcategory_id: &str,
  ) -> Result<CategoryNavbarResponseData, DBError> {
    category_navbar(self, ctx, category_id, subcategory_id).await
  }
  async fn products_category(
    &self,
    ctx: Arc<Context>,
    category_id: &str,
    subcategory_ids: &[String],
    page: u32,
    last_id: &str,
    limit: i64,
    sort_by: Option<&str>,
    sort_direction: Option<&str>,
  ) -> Result<Vec<ProductsCategoryItem>, DBError> {
    products_category(self, ctx, category_id, subcategory_ids, page, last_id, limit, sort_by, sort_direction).await
  }
}
