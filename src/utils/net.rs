use http::Uri;
use std::io::{Error, ErrorKind};

pub fn validate_url_target(url: &str) -> Result<Uri, Error> {
  url
    .parse::<Uri>()
    .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid URL: {}", e)))
}

