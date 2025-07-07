use std::sync::OnceLock;

use regex::Regex;
use unidecode::unidecode;

#[derive(Debug)]
pub struct Slug {
  pub lowercase: bool,
  pub replace_underscores: bool,
  pub trim: bool,
  pub max_length: Option<usize>,
}

impl Default for Slug {
  fn default() -> Self {
    Self { lowercase: true, replace_underscores: true, trim: true, max_length: None }
  }
}

// This matches any sequence of characters that are NOT letters or numbers.
//     [^...] - Negated character class (matches anything NOT in the class)
//     \p{L} - Any kind of letter from any language (Unicode property)
//     \p{N} - Any kind of numeric character (Unicode property)
//     + - One or more occurrences
fn non_alnum_regex() -> &'static Regex {
  static RE_NON_ALNUM: OnceLock<Regex> = OnceLock::new();
  RE_NON_ALNUM.get_or_init(|| Regex::new(r"[^\p{L}\p{N}_]+").unwrap())
}

// Purpose: After the first replacement, you might get multiple hyphens
// (like "Hello---World"). This collapses them to a single hyphen.
fn hyphens_regex() -> &'static Regex {
  static RE_HYPHENS: OnceLock<Regex> = OnceLock::new();
  RE_HYPHENS.get_or_init(|| Regex::new(r"-+").unwrap())
}

// This matches one or more underscores, used when you want to preserve underscores in slugs.
fn underscores_regex() -> &'static Regex {
  static RE_UNDERSCORES: OnceLock<Regex> = OnceLock::new();
  RE_UNDERSCORES.get_or_init(|| Regex::new(r"_+").unwrap())
}

fn all_non_alnum_regex() -> &'static Regex {
  static RE_ALL_NON_ALNUM: OnceLock<Regex> = OnceLock::new();
  RE_ALL_NON_ALNUM.get_or_init(|| Regex::new(r"[^\p{L}\p{N}]+").unwrap())
}

impl Slug {
  /// Input: " Café Zelda 2.0: Special_Edition! "
  /// Steps:
  ///   unidecode: " Cafe Zelda 2.0: Special_Edition! "
  ///   Replace non-alphanumeric (default): "--Cafe-Zelda-2-0--Special_Edition--"
  ///   Collapse hyphens: "-Cafe-Zelda-2-0-Special_Edition-"
  ///   Trim: "Cafe-Zelda-2-0-Special_Edition"
  ///   Lowercase: "cafe-zelda-2-0-special_edition"
  ///   Length limit (e.g., max=20): "cafe-zelda-2-0-spe"
  pub fn generate_slug(&self, input: &str) -> String {
    let mut slug = unidecode(input);

    // Replace special characters
    if self.replace_underscores {
      slug = all_non_alnum_regex().replace_all(&slug, "-").to_string();
    } else {
      // First handle underscores and spaces together
      slug = slug.replace(" _ ", "_").replace(" _", "_").replace("_ ", "_");
      // Then replace remaining special characters (except those already processed)
      slug = non_alnum_regex().replace_all(&slug, "-").to_string();
      // Collapse multiple underscores
      slug = underscores_regex().replace_all(&slug, "_").to_string();
    }

    // Collapse multiple hyphens
    slug = hyphens_regex().replace_all(&slug, "-").to_string();

    // Trim
    if self.trim {
      slug = slug.trim_matches('-').to_string();
      if !self.replace_underscores {
        slug = slug.trim_matches('_').to_string();
      }
    }

    // Lowercase
    if self.lowercase {
      slug = slug.to_lowercase();
    }

    if let Some(max) = self.max_length {
      if slug.len() > max {
        slug = slug.chars().take(max).collect();
      }
    }

    // Final trim — always clean up trailing hyphens/underscores after cutting
    if self.trim {
      slug = slug.trim_matches('-').to_string();
      if !self.replace_underscores {
        slug = slug.trim_matches('_').to_string();
      }
    }
    slug
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_slug() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("My _Fancy_ Product!"), "my-fancy-product");
  }

  #[test]
  fn test_preserve_underscores() {
    let slug = Slug { replace_underscores: false, ..Slug::default() };
    assert_eq!(slug.generate_slug("My _Fancy_ Product!"), "my_fancy_product");
  }

  #[test]
  fn test_max_length() {
    let slug = Slug { max_length: Some(10), ..Slug::default() };
    assert_eq!(slug.generate_slug("Very Long Product Name"), "very-long");
  }

  #[test]
  fn test_multiple_underscores() {
    let slug = Slug { replace_underscores: false, ..Slug::default() };
    assert_eq!(slug.generate_slug("A__B___C"), "a_b_c");
  }

  #[test]
  fn test_unicode_chars() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("Café au Lait"), "cafe-au-lait");
  }

  #[test]
  fn test_mixed_case() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("MixED CaSe"), "mixed-case");
  }

  #[test]
  fn test_numbers() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("Product 2023 v2"), "product-2023-v2");
  }

  #[test]
  fn test_special_chars() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("Hello@World#123"), "hello-world-123");
  }

  #[test]
  fn test_leading_trailing_special() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("!!Hello World!!"), "hello-world");
  }

  #[test]
  fn test_multiple_hyphens() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("Hello---World"), "hello-world");
  }

  #[test]
  fn test_empty_string() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug(""), "");
  }

  #[test]
  fn test_whitespace_only() {
    let slug = Slug::default();
    assert_eq!(slug.generate_slug("   "), "");
  }

  #[test]
  fn test_max_length_trim_vs_notrim() {
    let trimmed = Slug { max_length: Some(10), trim: true, ..Slug::default() };
    assert_eq!(trimmed.generate_slug("  Complex-Name___ "), "complex-na");

    let notrim = Slug { max_length: Some(10), trim: false, ..Slug::default() };
    assert_eq!(notrim.generate_slug("  Complex-Name___ "), "-complex-n");
  }
}
