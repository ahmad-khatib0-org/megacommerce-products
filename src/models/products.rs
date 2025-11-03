use lazy_static::lazy_static;
use regex::Regex;

pub static PRODUCT_TITLE_MIN_LENGTH: usize = 5;
pub static PRODUCT_TITLE_MAX_LENGTH: usize = 250;
pub static PRODUCT_DESCRIPTION_MIN_LENGTH: usize = 20;
pub static PRODUCT_DESCRIPTION_MAX_LENGTH: usize = 1024;
pub static PRODUCT_DESCRIPTION_BULLET_POINTS_MIN_LENGTH: usize = 1;
pub static PRODUCT_DESCRIPTION_BULLET_POINTS_MAX_LENGTH: usize = 30;
pub static PRODUCT_DESCRIPTION_BULLET_POINT_MIN_LENGTH: usize = 5;
pub static PRODUCT_DESCRIPTION_BULLET_POINT_MAX_LENGTH: usize = 255;
pub static PRODUCT_SKU_MIN_LENGTH: usize = 3;
pub static PRODUCT_SKU_MAX_LENGTH: usize = 60;
pub static PRODUCT_BRAND_NAME_MIN_LENGTH: usize = 3;
pub static PRODUCT_BRAND_NAME_MAX_LENGTH: usize = 60;
pub const PRODUCT_MINIMUM_INVENTORY_QUANTITY: u64 = 1;
pub const PRODUCT_OFFERING_CONDITION_NOTE_MIN_LENGTH: usize = 5;
pub const PRODUCT_OFFERING_CONDITION_NOTE_MAX_LENGTH: usize = 255;
pub const PRODUCT_MINIMUM_ORDER_MAX_OPTIONS: usize = 4;
pub const PRODUCT_MINIMUM_ORDER_MIN_OPTIONS: usize = 1;
pub const PRODUCT_VARIATION_TITLE_MIN_LENGTH: usize = 3;
pub const PRODUCT_VARIATION_TITLE_MAX_LENGTH: usize = 32;
pub static PRODUCT_MIN_IMAGES_COUNT: usize = 1;
pub static PRODUCT_MAX_IMAGES_COUNT: usize = 10;
pub static PRODUCT_MEDIA_MAX_ALLOWED_DIRECT_UPLOAD_SIZE_BYTES: usize = 1024 * 1024 * 40;
pub static PRODUCT_IMAGE_ACCEPTED_TYPES: [&str; 4] =
  ["image/png", "image/webp", "image/jpeg", "image/jpg"];
pub static PRODUCT_ID_TYPES: [&str; 4] = ["upc", "ean", "isbn", "gtin"];

pub enum ProductOfferingCondition {
  New,
  Used,
}

impl ProductOfferingCondition {
  pub const ALL_STR: [&'static str; 2] = ["new", "used"];

  pub fn as_slice() -> &'static [&'static str] {
    &Self::ALL_STR
  }

  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::New => "new",
      Self::Used => "used",
    }
  }

  pub fn from_str(value: &str) -> Self {
    match value {
      "new" => Self::New,
      "used" => Self::Used,
      _ => panic!("invalid value for ProductOfferingCondition"),
    }
  }

  pub fn as_vec() -> Vec<&'static str> {
    // manually list the variants
    [Self::New, Self::Used].iter().map(|c| c.as_str()).collect()
  }
}

pub enum ProductFulfillmentType {
  Megacommerce,
  Supplier,
}

impl ProductFulfillmentType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Megacommerce => "megacommerce",
      Self::Supplier => "supplier",
    }
  }
}

lazy_static! {
    // UPC (Universal Product Code) - Most common
    // UPC-A: 12 digits
    pub static ref PRODUCT_ID_TYPE_UPC_REGEX: Regex = Regex::new(r"^\d{12}$").unwrap();

    // EAN (European Article Number)
    // EAN-13: 13 digits
    pub static ref PRODUCT_ID_TYPE_EAN_REGEX: Regex = Regex::new(r"^\d{13}$").unwrap();

    // ISBN (International Standard Book Number) - For books
    // ISBN-10: 10 digits or 9 digits + X
    pub static ref PRODUCT_ID_TYPE_ISBN_REGEX: Regex = Regex::new(r"^(?:\d{9}[\dX]|\d{10})$").unwrap();

    // GTIN (Global Trade Item Number
    // GTIN: Can be 8, 12, 13, or 14 digits
    pub static ref PRODUCT_ID_TYPE_GTIN_REGEX: Regex = Regex::new(r"^\d{8,14}$").unwrap();
}

pub fn validate_upc(upc: &str) -> bool {
  PRODUCT_ID_TYPE_UPC_REGEX.is_match(upc) && validate_gtin_checksum(upc)
}

pub fn validate_ean(ean: &str) -> bool {
  PRODUCT_ID_TYPE_EAN_REGEX.is_match(ean) && validate_gtin_checksum(ean)
}

pub fn validate_isbn(isbn: &str) -> bool {
  PRODUCT_ID_TYPE_ISBN_REGEX.is_match(isbn) && validate_isbn_checksum(isbn)
}

pub fn validate_gtin(gtin: &str) -> bool {
  PRODUCT_ID_TYPE_GTIN_REGEX.is_match(gtin) && validate_gtin_checksum(gtin)
}

// GTIN checksum validation (used for UPC, EAN, GTIN)
fn validate_gtin_checksum(code: &str) -> bool {
  let digits: Vec<u32> = code.chars().filter_map(|c| c.to_digit(10)).collect();

  if digits.len() < 8 || digits.len() > 14 {
    return false;
  }

  let sum: u32 = digits
    .iter()
    .enumerate()
    .map(|(i, &digit)| {
      let multiplier = if (digits.len() - i) % 2 == 0 { 1 } else { 3 };
      digit * multiplier
    })
    .sum();

  sum % 10 == 0
}

// ISBN-10 checksum validation
fn validate_isbn_checksum(isbn: &str) -> bool {
  let clean_isbn = isbn.replace("-", "");
  if clean_isbn.len() != 10 {
    return false;
  }

  let sum: u32 = clean_isbn
    .chars()
    .enumerate()
    .map(|(i, c)| {
      let digit = match c {
        'X' | 'x' if i == 9 => 10,
        _ => c.to_digit(10).unwrap_or(0),
      };
      digit * (10 - i as u32)
    })
    .sum();

  sum % 11 == 0
}

// Usage example:
pub fn product_id_is_validate(id_type: &str, id_value: &str) -> bool {
  match id_type.to_lowercase().as_str() {
    "upc" => validate_upc(id_value),
    "ean" => validate_ean(id_value),
    "isbn" => validate_isbn(id_value),
    "gtin" => validate_gtin(id_value),
    _ => false,
  }
}

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

pub enum ProductCreateStepsNames {
  Identity,
  Description,
  Details,
  Media,
  Offer,
  Safety,
}

impl ProductCreateStepsNames {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Identity => "identity",
      Self::Description => "description",
      Self::Details => "details",
      Self::Media => "media",
      Self::Offer => "offer",
      Self::Safety => "safety",
    }
  }
}
