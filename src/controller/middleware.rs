use tonic::{metadata::MetadataValue, Request, Status};

use crate::models::network::Header;

pub(super) fn auth_middleware(req: Request<()>) -> Result<Request<()>, Status> {
  let token: MetadataValue<_> = "Bearer some-token".parse().unwrap();

  match req.metadata().get(Header::Authorization.as_str()) {
    Some(t) if token == t => Ok(req),
    _ => Err(Status::unauthenticated("no valid authentication token")),
  }
}
