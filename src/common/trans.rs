use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use megacommerce_proto::{TranslationElements, TranslationsGetRequest};
use megacommerce_shared::models::{
  context::Context,
  errors::{app_error_from_proto_app_error, BoxedErr, ErrorType, InternalError},
};
use tokio::time::timeout;
use tonic::Request;

use crate::common::main::Common;

impl Common {
  pub async fn translations_get(
    &mut self,
  ) -> Result<HashMap<String, TranslationElements>, Box<dyn Error>> {
    let err_msg = "failed to get translations from common service";
    let mk_err = |msg: &str, err: BoxedErr| {
      Box::new(InternalError {
        err_type: ErrorType::Internal,
        temp: false,
        err,
        msg: msg.into(),
        path: "products.common.translations_get".into(),
      })
    };

    let req = Request::new(TranslationsGetRequest {});
    let res = timeout(Duration::from_secs(5), self.client().unwrap().translations_get(req)).await;

    match res {
      Ok(Ok(res)) => {
        if res.get_ref().error.is_some() {
          let err = &res.get_ref().error.as_ref().unwrap();
          let ctx = Arc::new(Context::default());
          return Err(mk_err(err_msg, Box::new(app_error_from_proto_app_error(ctx, err))));
        } else {
          Ok(res.get_ref().data.clone())
        }
      }
      Err(e) => {
        return Err(mk_err("failed to get configurations: request timeout", Box::new(e)));
      }
      Ok(Err(e)) => {
        return Err(mk_err(err_msg, Box::new(e)));
      }
    }
  }
}
