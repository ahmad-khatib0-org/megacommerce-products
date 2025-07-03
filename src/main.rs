use std::error::Error;

use megacommerce_products::server::main::{Server, ServerArgs};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();
  tracing::subscriber::set_global_default(subscriber).expect("failed to set logger");

  let args = ServerArgs {};

  let server = Server::new(args).await;
  match server {
    Ok(mut srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}
