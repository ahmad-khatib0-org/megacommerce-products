use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

pub type TranslateFunc =
  Box<dyn Fn(&str, &str, &HashMap<String, Value>) -> Result<String, Box<dyn Error>>>;
