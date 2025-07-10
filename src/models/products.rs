pub static PRODUCT_TITLE_MIN_LENGTH: usize = 5;
pub static PRODUCT_TITLE_MAX_LENGTH: usize = 250;
pub static PRODUCT_DESCRIPTION_MIN_LENGTH: usize = 20;
pub static PRODUCT_DESCRIPTION_MAX_LENGTH: usize = 1024;
pub static PRODUCT_SKU_MIN_LENGTH: usize = 3;
pub static PRODUCT_SKU_MAX_LENGTH: usize = 60;

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
