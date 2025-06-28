use std::sync::Arc;

use tokio::spawn;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::Mutex;

use crate::models::config::Config;
use crate::models::errors::InternalError;

pub struct Server {
  pub errors: mpsc::Sender<InternalError>,
  pub config: Arc<Mutex<Config>>,
}

#[derive(Debug)]
pub struct ServerArgs {}

impl Server {
  pub async fn new(_: ServerArgs) -> Self {
    let (tx, rx) = mpsc::channel::<InternalError>(100);

    let server = Self {
      errors: tx,
      config: Arc::new(Mutex::new(Config::default())),
    };

    server.init_service_config().await;

    let err_rx = rx;
    let _ = server.errors.clone();
    spawn(async move {
      Server::errors_listener(err_rx).await;
    });

    server
  }

  async fn errors_listener(mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("{}", msg)
    }
  }
}
