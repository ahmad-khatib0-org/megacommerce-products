use std::error::Error;

use megacommerce_products::server::main::{Server, ServerArgs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let args = ServerArgs {};

  let server = Server::new(args).await;
  match server {
    Ok(srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}
