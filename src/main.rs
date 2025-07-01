use std::error::Error;

use megacommerce_products::server::main::{Server, ServerArgs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let args = ServerArgs {};
  let _ = Server::new(args).await;

  Ok(())
}
