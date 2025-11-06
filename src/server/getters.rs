use megacommerce_proto::Config;
use megacommerce_shared::models::r_lock::RLock;
use sqlx::{Pool, Postgres};

use crate::server::{object_storage::ObjectStorage, Server};

impl Server {
  /// Return a read-only config to pass downstream
  pub fn config(&self) -> RLock<Config> {
    RLock::<Config>(self.shared_config.clone())
  }

  /// Return a read-only postgres database instance to pass downstream
  pub fn db(&self) -> RLock<Pool<Postgres>> {
    RLock::<Pool<Postgres>>(self.db.as_ref().unwrap().clone())
  }

  /// Return a read-only Object Storage instance to pass downstream
  pub fn object_storage(&self) -> RLock<ObjectStorage> {
    RLock::<ObjectStorage>(self.object_storage.as_ref().unwrap().clone())
  }
}
