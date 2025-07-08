use std::{error::Error, sync::Arc};

use megacommerce_proto::ProductTag;

#[derive(Debug, Clone)]
pub struct Cache {
  tags: Arc<Vec<ProductTag>>,
}

impl Cache {
  pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
    let cache = Self { tags: Arc::new(vec![]) };

    Ok(cache)
  }

  pub fn tags(&self) -> Arc<Vec<ProductTag>> {
    self.tags.clone()
  }
}
