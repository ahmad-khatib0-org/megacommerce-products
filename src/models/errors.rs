use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use derive_more::Display;
use megacommerce_proto::{AppError as AppErrorProto, NestedStringMap, StringMap};

use crate::models::{
  context::Context,
  trans::{tr, TranslateFunc},
};

const MAX_ERROR_LENGTH: usize = 1024;
const NO_TRANSLATION: &str = "<untranslated>";

#[derive(Debug, Display)]
#[display("InternalError: {} {} {} {}", temp, err, msg, path)]
pub struct InternalError {
  pub temp: bool,
  pub err: Box<dyn Error + Send + Sync>,
  pub msg: String,
  pub path: String,
}

impl Error for InternalError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(&*self.err)
  }
}

#[derive(Debug)]
pub struct AppError {
  pub ctx: Arc<Context>,
  pub id: String,
  pub message: String,
  pub detailed_error: String,
  pub request_id: Option<String>,
  pub status_code: Option<i32>,
  pub tr_params: Option<HashMap<String, serde_json::Value>>,
  pub params: HashMap<String, String>,
  pub nested_params: HashMap<String, HashMap<String, String>>,
  pub where_: String,
  pub skip_translation: bool,
  pub wrapped: Option<Box<dyn Error + Send + Sync>>,
}

impl AppError {
  pub fn new(
    ctx: Arc<Context>,
    where_: impl Into<String>,
    id: impl Into<String>,
    tr_params: Option<HashMap<String, serde_json::Value>>,
    details: impl Into<String>,
    status_code: Option<i32>,
    wrapped: Option<Box<dyn Error + Send + Sync>>,
  ) -> Self {
    let mut err = Self {
      ctx,
      id: id.into(),
      message: "".to_string(),
      detailed_error: details.into(),
      request_id: None,
      status_code,
      tr_params,
      params: HashMap::new(),
      nested_params: HashMap::new(),
      where_: where_.into(),
      skip_translation: false,
      wrapped,
    };

    let boxed_tr = Box::new(|lang: &str, id: &str, params: &HashMap<String, serde_json::Value>| {
      let params_option = if params.is_empty() { None } else { Some(params.clone()) };
      tr(lang, id, params_option).map_err(|e| Box::new(e) as Box<dyn Error>)
    });

    err.translate(Some(boxed_tr));
    err
  }

  pub fn error_string(&self) -> String {
    let mut s = String::new();

    if !self.where_.is_empty() {
      s.push_str(&self.where_);
      s.push_str(": ");
    }

    if self.message != NO_TRANSLATION {
      s.push_str(&self.message);
    }

    if !self.detailed_error.is_empty() {
      if self.message != NO_TRANSLATION {
        s.push_str(", ");
      }
      s.push_str(&self.detailed_error);
    }

    if let Some(ref wrapped) = self.wrapped {
      s.push_str(", ");
      s.push_str(&wrapped.to_string());
    }

    if s.len() > MAX_ERROR_LENGTH {
      s.truncate(MAX_ERROR_LENGTH);
      s.push_str("...");
    }

    s
  }

  pub fn translate(&mut self, tf: Option<TranslateFunc>) {
    if self.skip_translation {
      return;
    }

    if let Some(tf) = tf {
      let empty = HashMap::new();
      let params = self.tr_params.as_ref().unwrap_or(&empty);
      if let Ok(translated) = tf(&self.ctx.accept_language, &self.id, params) {
        self.message = translated;
        return;
      }
    }
    self.message = self.id.clone();
  }

  pub fn unwrap(&self) -> Option<&(dyn Error + Send + Sync)> {
    self.wrapped.as_deref()
  }

  pub fn wrap(mut self, err: Box<dyn Error + Send + Sync>) -> Self {
    self.wrapped = Some(err);
    self
  }

  pub fn wipe_detailed(&mut self) {
    self.wrapped = None;
    self.detailed_error.clear();
  }

  pub fn default() -> Self {
    Self {
      ctx: Arc::new(Context::default()),
      id: String::new(),
      message: String::new(),
      detailed_error: String::new(),
      request_id: None,
      status_code: None,
      tr_params: None,
      params: HashMap::new(),
      nested_params: HashMap::new(),
      where_: String::new(),
      skip_translation: false,
      wrapped: None,
    }
  }

  // Convert to proto-generated struct (replace with your proto types)
  pub fn to_proto(&self) -> AppErrorProto {
    let mut nested = HashMap::with_capacity(self.nested_params.len());
    for (k, v) in &self.nested_params {
      nested.insert(k.clone(), StringMap { data: v.clone() });
    }

    AppErrorProto {
      id: self.id.clone(),
      message: self.message.clone(),
      detailed_error: self.detailed_error.clone(),
      status_code: self.status_code.unwrap_or(0) as i32,
      r#where: self.where_.clone(),
      skip_translation: self.skip_translation,
      params: Some(StringMap { data: self.params.clone() }),
      nested_params: Some(NestedStringMap { data: nested }),
      request_id: self.request_id.clone().unwrap_or_default(),
    }
  }
}

// Convert proto params to HashMaps
pub fn convert_proto_params(
  ae: &AppErrorProto,
) -> (HashMap<String, String>, HashMap<String, HashMap<String, String>>) {
  let mut shallow = HashMap::new();
  let mut nested = HashMap::new();

  if let Some(ref p) = ae.params {
    shallow.extend(p.data.clone());
  }
  if let Some(ref n) = ae.nested_params {
    for (k, v) in &n.data {
      nested.insert(k.clone(), v.data.clone());
    }
  }

  (shallow, nested)
}

// Convert from proto-generated struct
pub fn app_error_from_proto_app_error(ctx: Arc<Context>, ae: &AppErrorProto) -> AppError {
  let (params, nested) = convert_proto_params(ae);

  AppError {
    ctx,
    id: ae.id.clone(),
    message: ae.message.clone(),
    detailed_error: ae.detailed_error.clone(),
    request_id: if ae.request_id.is_empty() { None } else { Some(ae.request_id.clone()) },
    status_code: Some(ae.status_code as i32),
    tr_params: None,
    params,
    nested_params: nested,
    where_: ae.r#where.clone(),
    skip_translation: ae.skip_translation,
    wrapped: None,
  }
}

// Implement std::fmt::Display for error formatting
impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.error_string())
  }
}

impl Error for AppError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    self.wrapped.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static))
  }
}
