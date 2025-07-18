use megacommerce_proto::{
  products_service_server::ProductsService, ProductCreateRequest, ProductCreateResponse,
  ProductDataRequest, ProductDataResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::Controller;

#[tonic::async_trait]
impl ProductsService for Controller {
  async fn product_create(
    &self,
    req: Request<ProductCreateRequest>,
  ) -> Result<Response<ProductCreateResponse>, Status> {
    crate::controller::product_create::product_create(self, req).await
  }

  async fn product_data(
    &self,
    req: Request<ProductDataRequest>,
  ) -> Result<Response<ProductDataResponse>, Status> {
    // Delegate to product_data.rs implementation
    crate::controller::product_data::product_data(self, req).await
  }
}
