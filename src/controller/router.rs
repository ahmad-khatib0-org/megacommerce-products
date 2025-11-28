use megacommerce_proto::{
  products_service_server::ProductsService, BestSellingProductsRequest,
  BestSellingProductsResponse, BigDiscountProductsRequest, BigDiscountProductsResponse,
  NewlyAddedProductsRequest, NewlyAddedProductsResponse, ProductCreateRequest,
  ProductCreateResponse, ProductDataRequest, ProductDataResponse, ProductListRequest,
  ProductListResponse, ProductSnapshotRequest, ProductSnapshotResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{
  best_selling_products::best_selling_products, big_discount_products::big_discount_products,
  newly_added_products::newly_added_products, product_create::product_create,
  product_data::product_data, product_list::product_list, product_snapshot::product_snapshot,
  Controller,
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
  async fn product_list(
    &self,
    req: Request<ProductListRequest>,
  ) -> Result<Response<ProductListResponse>, Status> {
    product_list(self, req).await
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
}
