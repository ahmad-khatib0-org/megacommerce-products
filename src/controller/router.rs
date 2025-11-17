use megacommerce_proto::{
  products_service_server::ProductsService, ProductCreateRequest, ProductCreateResponse,
  ProductDataRequest, ProductDataResponse, ProductListRequest, ProductListResponse,
  ProductSnapshotRequest, ProductSnapshotResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{
  product_create::product_create, product_data::product_data, product_list::product_list,
  product_snapshot::product_snapshot, Controller,
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
}
