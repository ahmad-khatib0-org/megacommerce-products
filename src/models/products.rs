#[derive(Debug)]
pub enum ProductStatus {
  Draft,
  Pending,
  Published,
}

impl ProductStatus {
  pub fn as_string(&self) -> String {
    match self {
      ProductStatus::Draft => "draft".to_string(),
      ProductStatus::Pending => "pending".to_string(),
      ProductStatus::Published => "published".to_string(),
    }
  }
}
