pub mod dbstore;

use megacommerce_proto::{Product, ProductListItem, ProductListRequest};
use megacommerce_shared::{models::context::Context, store::errors::DBError};
use std::{fmt, sync::Arc};

#[tonic::async_trait]
pub trait ProductsStore: fmt::Debug + Send + Sync {
  async fn product_create(&self, ctx: Arc<Context>, product: &Product) -> Result<(), DBError>;
  async fn product_list(
    &self,
    ctx: Arc<Context>,
    request: &ProductListRequest,
  ) -> Result<Vec<ProductListItem>, DBError>;
}
