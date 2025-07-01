use megacommerce_proto::{products_service_server, Config as SharedConfig};
use std::error::Error;
use tonic::{transport::Server as GrpcServer, Request, Response, Status};

#[derive(Debug)]
struct Controller {
  cfg: Option<SharedConfig>,
}

#[derive(Debug)]
struct ControllerArgs {}

impl Controller {
  pub fn new(args: ControllerArgs) -> Controller {
    let ctr = Controller { cfg: None };

    ctr
  }

  // pub fn run() -> Result<(), Box<dyn Error>> {}
}
