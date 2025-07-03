use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex, OnceLock};

use megacommerce_proto::TranslationElements;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error as ThisError;

pub type TranslateFunc =
  Box<dyn Fn(&str, &str, &HashMap<String, Value>) -> Result<String, Box<dyn Error>>>;

fn parse_translations_grpc_respones(
  data: HashMap<String, TranslationElements>,
) -> HashMap<String, HashMap<String, String>> {
  let mut result = HashMap::new();
  for (lang, elements) in data {
    let mut lang_map = HashMap::new();
    for el in elements.trans {
      lang_map.insert(el.id, el.tr);
    }

    result.insert(lang, lang_map);
  }

  result
}

#[derive(Debug, ThisError)]
pub enum TranslationError {
  #[error("translation store is not initialized")]
  NotInitialized,
  #[error("translation is missing params")]
  MissingParams,
  #[error("translation key is not found: {0}")]
  KeyNotFound(String),
  #[error("template render error: {0}")]
  RenderError(String),
}

#[derive(Debug)]
struct TemplatePool {
  available: VecDeque<tera::Tera>,
  template_str: String,
  has_vars: bool,
  max_size: usize,
}

impl From<tera::Error> for TranslationError {
  fn from(value: tera::Error) -> Self {
    TranslationError::RenderError(value.to_string())
  }
}

impl TemplatePool {
  fn new(template: &str, max_size: usize) -> Self {
    Self {
      available: VecDeque::with_capacity(max_size),
      template_str: template.to_string(),
      has_vars: template.contains("{{") && template.contains("}}"),
      max_size,
    }
  }

  fn get(&mut self) -> Result<tera::Tera, tera::Error> {
    self.available.pop_front().map_or_else(
      || {
        let mut t = tera::Tera::default();
        t.add_raw_template("pooled_template", &self.template_str)?;
        Ok(t)
      },
      Ok,
    )
  }

  fn return_instance(&mut self, instance: tera::Tera) {
    if self.available.len() < self.max_size {
      self.available.push_back(instance);
    }
    // Else dropped when pool is full
  }
}

static TRANSLATION_STORE: OnceLock<HashMap<String, HashMap<String, Arc<Mutex<TemplatePool>>>>> =
  OnceLock::new();

pub fn translations_init(
  trans: HashMap<String, TranslationElements>,
  max_pool_size: usize,
) -> Result<(), TranslationError> {
  let parsed = parse_translations_grpc_respones(trans);
  let mut store = HashMap::new();

  for (lang, lang_trans) in parsed {
    let mut lang_map = HashMap::new();
    for (id, tr) in lang_trans {
      let pool = Arc::new(Mutex::new(TemplatePool::new(&tr, max_pool_size)));
      lang_map.insert(id, pool);
    }
    store.insert(lang, lang_map);
  }

  TRANSLATION_STORE.set(store).map_err(|_| TranslationError::NotInitialized)
}

pub fn tr<P: Serialize>(
  lang: &str,
  id: &str,
  params: Option<P>,
) -> Result<String, TranslationError> {
  let store = TRANSLATION_STORE.get().ok_or(TranslationError::NotInitialized)?;

  let pool = store
    .get(lang)
    .and_then(|lang_pools| lang_pools.get(id))
    .ok_or_else(|| TranslationError::KeyNotFound(id.to_string()))?;

  let mut pool_guard = pool.lock().unwrap();
  if pool_guard.has_vars && params.is_none() {
    return Err(TranslationError::MissingParams);
  }

  let tera = pool_guard.get()?;

  let result = match params {
    Some(p) => {
      let context = tera::Context::from_serialize(&p)?;
      tera.render("pooled_template", &context)
    }
    None => {
      // For non-parameterized templates, just return the raw template string
      Ok(pool_guard.template_str.clone())
    }
  }?;

  pool_guard.return_instance(tera);
  Ok(result)
}
