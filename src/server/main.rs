use std::error::Error;
use std::sync::Arc;

use megacommerce_proto::Config as SharedConfig;
use tokio::spawn;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::Mutex;

use crate::common::main::{Common, CommonArgs};
use crate::controller::main::{Controller, ControllerArgs};
use crate::models::config::Config as ServiceConfig;
use crate::models::errors::InternalError;

pub struct Server {
  pub(crate) errors: mpsc::Sender<InternalError>,
  pub(crate) common: Option<Common>,
  pub(crate) service_config: Arc<Mutex<ServiceConfig>>,
  pub(crate) shared_config: Arc<Mutex<SharedConfig>>,
}

#[derive(Debug)]
pub struct ServerArgs {}

impl Server {
  pub async fn new(_: ServerArgs) -> Result<Self, Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<InternalError>(100);

    let mut server = Self {
      errors: tx,
      common: None,
      service_config: Arc::new(Mutex::new(ServiceConfig::default())),
      shared_config: Arc::new(Mutex::new(SharedConfig::default())),
    };

    server.init_service_config().await?;

    let common_args = {
      let service_config = server.service_config.lock().await.clone();
      CommonArgs { service_config }
    };

    match Common::new(common_args).await {
      Ok(com) => server.common = Some(com),
      Err(err) => return Err(err),
    };

    let err_rx = rx;
    spawn(async move {
      Server::errors_listener(err_rx).await;
    });

    Ok(server)
  }

  pub async fn run(&self) -> Result<(), Box<dyn Error>> {
    let ctr_args = {
      let cfg = self.shared_config.lock().await.clone();
      ControllerArgs { cfg }
    };

    let controller = Controller::new(ctr_args);
    controller.run().await
  }

  async fn errors_listener(mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("from here {}", msg)
    }
  }
}
