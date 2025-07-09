pub mod dbstore;
pub mod errors;

use megacommerce_proto::Product;
use std::{fmt, sync::Arc};

use crate::{models::context::Context, store::database::errors::DBError};

#[tonic::async_trait]
pub trait ProductsStore: fmt::Debug + Send + Sync {
  async fn product_create(&self, ctx: Arc<Context>, product: &Product) -> Result<(), DBError>;
}
