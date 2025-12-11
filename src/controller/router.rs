use megacommerce_proto::{
  products_service_server::ProductsService, BestSellingProductsRequest,
  BestSellingProductsResponse, BigDiscountProductsRequest, BigDiscountProductsResponse,
  CategoryNavbarRequest, CategoryNavbarResponse, HeroProductsRequest, HeroProductsResponse,
  NewlyAddedProductsRequest, NewlyAddedProductsResponse, ProductCreateRequest,
  ProductCreateResponse, ProductDataRequest, ProductDataResponse, ProductDetailsRequest,
  ProductDetailsResponse, ProductSnapshotRequest, ProductSnapshotResponse, ProductsCategoryRequest,
  ProductsCategoryResponse, ProductsToLikeRequest, ProductsToLikeResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{
  best_selling_products::best_selling_products, big_discount_products::big_discount_products,
  category_navbar::category_navbar, hero_products::hero_products,
  newly_added_products::newly_added_products, product_create::product_create,
  product_data::product_data, product_details::product_details, product_snapshot::product_snapshot,
  products_category::products_category, products_to_like::products_to_like, Controller,
};

#[tonic::async_trait]
impl ProductsService for Controller {
  async fn product_create(
    &self,
    req: Request<ProductCreateRequest>,
  ) -> Result<Response<ProductCreateResponse>, Status> {
    product_create(self, req).await
  }
  async fn product_data(
    &self,
    req: Request<ProductDataRequest>,
  ) -> Result<Response<ProductDataResponse>, Status> {
    product_data(self, req).await
  }
  async fn products_to_like(
    &self,
    req: Request<ProductsToLikeRequest>,
  ) -> Result<Response<ProductsToLikeResponse>, Status> {
    products_to_like(self, req).await
  }
  async fn product_snapshot(
    &self,
    req: Request<ProductSnapshotRequest>,
  ) -> Result<Response<ProductSnapshotResponse>, Status> {
    product_snapshot(self, req).await
  }
  async fn best_selling_products(
    &self,
    req: Request<BestSellingProductsRequest>,
  ) -> Result<Response<BestSellingProductsResponse>, Status> {
    best_selling_products(self, req).await
  }
  async fn big_discount_products(
    &self,
    req: Request<BigDiscountProductsRequest>,
  ) -> Result<Response<BigDiscountProductsResponse>, Status> {
    big_discount_products(self, req).await
  }
  async fn newly_added_products(
    &self,
    req: Request<NewlyAddedProductsRequest>,
  ) -> Result<Response<NewlyAddedProductsResponse>, Status> {
    newly_added_products(self, req).await
  }
  async fn hero_products(
    &self,
    req: Request<HeroProductsRequest>,
  ) -> Result<Response<HeroProductsResponse>, Status> {
    hero_products(self, req).await
  }
  async fn product_details(
    &self,
    req: Request<ProductDetailsRequest>,
  ) -> Result<Response<ProductDetailsResponse>, Status> {
    product_details(self, req).await
  }
  async fn category_navbar(
    &self,
    req: Request<CategoryNavbarRequest>,
  ) -> Result<Response<CategoryNavbarResponse>, Status> {
    category_navbar(self, req).await
  }
  async fn products_category(
    &self,
    req: Request<ProductsCategoryRequest>,
  ) -> Result<Response<ProductsCategoryResponse>, Status> {
    products_category(self, req).await
  }
}
