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

  pub fn as_slice() -> [&'static str; 2] {
    ["megacommerce", "supplier"]
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

// Regexes expect digit-only strings. We will clean the input before testing.
lazy_static! {
    // UPC-A: 12 digits
    static ref PRODUCT_ID_TYPE_UPC_REGEX: Regex = Regex::new(r"^\d{12}$").unwrap();

    // EAN-13: 13 digits
    static ref PRODUCT_ID_TYPE_EAN_REGEX: Regex = Regex::new(r"^\d{13}$").unwrap();

    // ISBN-10: 10 chars, last may be X or x (we'll check after cleaning hyphens)
    static ref PRODUCT_ID_TYPE_ISBN10_REGEX: Regex = Regex::new(r"^(?:\d{9}[\dXx])$").unwrap();

    // GTIN: 8..14 digits
    static ref PRODUCT_ID_TYPE_GTIN_REGEX: Regex = Regex::new(r"^\d{8,14}$").unwrap();
}

/// Keep only ASCII digits from the input.
fn only_digits(input: &str) -> String {
  input.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// GTIN checksum validation (for UPC, EAN, GTIN, and ISBN-13).
/// We iterate from the RIGHTmost digit and apply weights 1,3,1,3...
fn validate_gtin_checksum(code: &str) -> bool {
  let digits: Vec<u32> = code.chars().filter_map(|c| c.to_digit(10)).collect();

  if digits.len() < 8 || digits.len() > 14 {
    return false;
  }

  // Iterate from right to left: index 0 is rightmost digit.
  let sum: u32 = digits
    .iter()
    .rev()
    .enumerate()
    .map(|(i, &digit)| {
      let multiplier = if i % 2 == 0 { 1 } else { 3 };
      digit * multiplier
    })
    .sum();

  sum % 10 == 0
}

pub fn validate_upc(upc: &str) -> bool {
  let cleaned = only_digits(upc);
  if !PRODUCT_ID_TYPE_UPC_REGEX.is_match(&cleaned) {
    return false;
  }
  validate_gtin_checksum(&cleaned)
}

pub fn validate_ean(ean: &str) -> bool {
  let cleaned = only_digits(ean);
  if !PRODUCT_ID_TYPE_EAN_REGEX.is_match(&cleaned) {
    return false;
  }
  validate_gtin_checksum(&cleaned)
}

pub fn validate_gtin(gtin: &str) -> bool {
  let cleaned = only_digits(gtin);
  if !PRODUCT_ID_TYPE_GTIN_REGEX.is_match(&cleaned) {
    return false;
  }
  validate_gtin_checksum(&cleaned)
}

/// Validate ISBN: accept hyphens/spaces. Support ISBN-10 and ISBN-13.
pub fn validate_isbn(isbn: &str) -> bool {
  // Remove spaces and hyphens (and keep letters for ISBN-10 final X).
  let no_hyphen_space: String = isbn.chars().filter(|c| !c.is_whitespace() && *c != '-').collect();

  // Check for ISBN-13 (13 digits) -> use GTIN/EAN logic
  let digits_only = only_digits(&no_hyphen_space);
  if digits_only.len() == 13 && PRODUCT_ID_TYPE_EAN_REGEX.is_match(&digits_only) {
    return validate_gtin_checksum(&digits_only);
  }

  // Check for ISBN-10 form: allow final X/x
  let isbn10_filtered: String =
    no_hyphen_space.chars().filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x').collect();

  if isbn10_filtered.len() == 10 && PRODUCT_ID_TYPE_ISBN10_REGEX.is_match(&isbn10_filtered) {
    // ISBN-10 checksum: sum(digit * (10 - index)) where final X == 10
    let sum: u32 = isbn10_filtered
      .chars()
      .enumerate()
      .map(|(i, c)| {
        let value = if i == 9 && (c == 'X' || c == 'x') { 10 } else { c.to_digit(10).unwrap_or(0) };
        value * (10 - i as u32)
      })
      .sum();
    return sum % 11 == 0;
  }

  false
}

pub fn product_id_is_validate(id_type: &str, id_value: &str) -> bool {
  match id_type.to_lowercase().as_str() {
    "upc" => validate_upc(id_value),
    "ean" => validate_ean(id_value),
    "isbn" => validate_isbn(id_value),
    "gtin" => validate_gtin(id_value),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn upc_example_valid() {
    // Your example: 036000291452 (12 digits UPC)
    assert!(product_id_is_validate("upc", "036000291452"));
  }

  #[test]
  fn isbn13_hyphenated_valid() {
    // Your example: 978-1-56619-909-4 (ISBN-13 hyphenated)
    assert!(product_id_is_validate("isbn", "978-1-56619-909-4"));
  }

  #[test]
  fn isbn10_valid_with_x() {
    // example ISBN-10 with X checksum (0306406152 is a classic valid ISBN-10)
    assert!(product_id_is_validate("isbn", "0306406152"));
  }

  #[test]
  fn gtin_8_valid_example() {
    // example GTIN-8: 96385074 (classic test value)
    assert!(product_id_is_validate("gtin", "96385074"));
  }

  #[test]
  fn invalid_examples() {
    assert!(!product_id_is_validate("upc", "1234567890123")); // wrong length
    assert!(!product_id_is_validate("isbn", "978-1-56619-909-0")); // bad checksum
  }
}
