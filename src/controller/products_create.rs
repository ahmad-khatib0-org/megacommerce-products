use megacommerce_proto::{
  products_service_server::ProductsService, ProductCreateRequest, ProductCreateResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::main::Controller;

#[tonic::async_trait]
impl ProductsService for Controller {
  async fn product_create(
    &self,
    _req: Request<ProductCreateRequest>,
  ) -> Result<Response<ProductCreateResponse>, Status> {
    let response = ProductCreateResponse {};
    Ok(Response::new(response))
  }
}
