use std::{error::Error, sync::RwLockReadGuard};

use megacommerce_proto::ProductTag;
use sqlx::query;

use crate::store::{cache::Cache, database::errors::handle_db_error};

impl Cache {
  pub fn tags(&self) -> Vec<ProductTag> {
    self.tags.read().unwrap().clone()
  }

  pub fn tags_as_ref(&self) -> RwLockReadGuard<'_, Vec<ProductTag>> {
    self.tags.read().unwrap()
  }

  pub(super) async fn tags_init(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
    let rows = query!(r#" SELECT id, name FROM tags "#)
      .fetch_all(self.db.as_ref())
      .await
      .map_err(|err| handle_db_error(err, "products.store.tags_init"))?;

    let mut tags = self.tags.write().unwrap();
    *tags = rows
      .into_iter()
      .map(|row| ProductTag { id: Some(row.id as u32), name: Some(row.name) })
      .collect();

    Ok(())
  }
}
