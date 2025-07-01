use std::{
  io::{Error, ErrorKind::InvalidInput},
  net::SocketAddr,
};

pub fn validate_url_target(url: &str) -> Result<SocketAddr, Error> {
  url
    .parse::<SocketAddr>()
    .map_err(|e| Error::new(InvalidInput, format!("invalid provided address {}", e)))
}
