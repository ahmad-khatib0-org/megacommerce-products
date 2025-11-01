use std::{error::Error, sync::Arc, time::Duration};

use megacommerce_shared::models::errors::{ErrorType, InternalError};
use sqlx::postgres::PgPoolOptions;

use crate::server::Server;

impl Server {
  pub(super) async fn init_database(&mut self) -> Result<(), Box<dyn Error>> {
    let cfg = self.shared_config.lock().await.sql.clone().unwrap();

    let db = PgPoolOptions::new()
      .max_connections(cfg.max_open_conns().clone() as u32)
      .min_connections(cfg.max_idle_conns() as u32)
      .max_lifetime(Duration::from_millis(cfg.conn_max_lifetime_milliseconds() as u64))
      .idle_timeout(Duration::from_millis(cfg.conn_max_idle_time_milliseconds() as u64))
      .connect(cfg.data_source())
      .await
      .map_err(|e| InternalError {
        temp: false,
        err_type: ErrorType::DBConnectionError,
        err: Box::new(e),
        msg: "failed to connect to database".into(),
        path: "products.server.init_database".into(),
      })?;

    self.db = Some(Arc::new(db));

    Ok(())
  }
}
