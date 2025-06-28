use std::error::Error;

use megacommerce_products::server::server::{Server, ServerArgs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let _ = Server::new(ServerArgs {}).await;

  Ok(())
}
