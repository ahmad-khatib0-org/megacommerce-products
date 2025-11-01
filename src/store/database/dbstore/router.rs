use std::sync::Arc;

use megacommerce_proto::{Product, ProductListItem, ProductListRequest};
use megacommerce_shared::{models::context::Context, store::errors::DBError};

use crate::store::database::{
  dbstore::{product_create::product_create, product_list::product_list, ProductsStoreImpl},
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
}
