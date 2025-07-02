use std::{error::Error, time::Duration};

use megacommerce_proto::{config_get_response, ConfigGetRequest};
use tokio::time::timeout;
use tonic::Request;

use crate::models::errors::InternalError;

use super::main::Common;

impl Common {
  pub async fn config_get(&mut self) -> Result<(), Box<dyn Error>> {
    let err_msg = "failed to get configurations from common service";
    let mk_err = |msg: &str, err: Box<dyn Error + Send + Sync>| {
      Box::new(InternalError {
        temp: false,
        err,
        msg: msg.into(),
        path: "products.common.config_get".into(),
      })
    };

    let req = Request::new(ConfigGetRequest {});
    let res = timeout(Duration::from_secs(5), self.client().unwrap().config_get(req)).await;

    match res {
      Ok(res) => match res?.into_inner().response {
        Some(config_get_response::Response::Data(res)) => {
          println!("got the config");
        }
        Some(config_get_response::Response::Error(res)) => {
          // return Err(mk_err(err_msg, Box::new(res)));
        }
        None => {
          return Err(mk_err("missing response field in config_get", "empty".into()));
        }
      },
      Err(e) => {
        return Err(mk_err("failed to get configurations: request timeout", Box::new(e)));
      }
      Ok(Err(e)) => {}
    }

    Ok(())
  }
}
