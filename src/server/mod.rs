mod config;
mod database;
mod getters;
pub mod object_storage;

use std::error::Error;
use std::sync::Arc;

use megacommerce_proto::Config as SharedConfig;
use megacommerce_shared::models::errors::{BoxedErr, ErrorType, InternalError};
use megacommerce_shared::models::translate::translations_init;
use sqlx::{Pool, Postgres};
use tokio::spawn;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::{Mutex, RwLock};

use crate::common::main::{Common, CommonArgs};
use crate::controller::{Controller, ControllerArgs};
use crate::models::config::Config as ServiceConfig;
use crate::server::object_storage::ObjectStorage;
use crate::store::cache::{Cache, CacheArgs};
use crate::store::database::dbstore::{ProductsStoreImpl, ProductsStoreImplArgs};

pub struct Server {
  pub(crate) errors: mpsc::Sender<InternalError>,
  pub(crate) db: Option<Arc<RwLock<Pool<Postgres>>>>,
  pub(crate) common: Option<Common>,
  pub(crate) service_config: Arc<Mutex<ServiceConfig>>,
  pub(crate) shared_config: Arc<RwLock<SharedConfig>>,
  pub(crate) object_storage: Option<Arc<RwLock<ObjectStorage>>>,
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
      shared_config: Arc::new(RwLock::new(SharedConfig::default())),
      db: None,
      object_storage: None,
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

    match server.common.as_mut().unwrap().config_get().await {
      Ok(cfg) => {
        let mut shared_config = server.shared_config.write().await;
        *shared_config = cfg;
      }
      Err(err) => return Err(err),
    }

    let err_rx = rx;
    spawn(async move {
      Server::errors_listener(err_rx).await;
    });

    Ok(server)
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    let mk_err = |msg: &str, err: BoxedErr| InternalError {
      temp: false,
      err_type: ErrorType::Internal,
      err,
      msg: msg.to_string(),
      path: "products.server.run".into(),
    };

    self.init_database().await?;
    self.object_storage = Some(Arc::new(RwLock::new(ObjectStorage::new(self.config()).await?)));

    let cache_args = CacheArgs { db: self.db() };
    let cache =
      Arc::new(Cache::new(cache_args).await.map_err(|e| mk_err("failed to initialize cache", e))?);

    let store_args = ProductsStoreImplArgs { db: self.db() };
    let store = Arc::new(ProductsStoreImpl::new(store_args));

    match self.common.as_mut().unwrap().translations_get().await {
      Ok(res) => {
        translations_init(res, 5)
          .map_err(|e| mk_err("failed to init translations", Box::new(e)))?;
      }
      Err(err) => return Err(err),
    }

    let ctr_args =
      ControllerArgs { cfg: self.config(), cache, store, storage: self.object_storage() };
    let controller = Controller::new(ctr_args);
    controller.run().await
  }

  async fn errors_listener(mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("from here {}", msg)
    }
  }
}
