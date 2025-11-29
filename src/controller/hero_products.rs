use std::sync::Arc;

use megacommerce_proto::{
  hero_products_response::Response::{Data, Error},
  HeroProductsRequest, HeroProductsResponse, HeroProductsResponseData,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, MSG_ID_ERR_INTERNAL},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::Controller;

pub async fn hero_products(
  c: &Controller,
  req: Request<HeroProductsRequest>,
) -> Result<Response<HeroProductsResponse>, Status> {
  let ctx = req.extensions().get::<Arc<Context>>().cloned().unwrap();
  let _req = req.into_inner();
  let path = "products.controller.hero_products";
  let return_err = |e: AppError| {
    return Response::new(HeroProductsResponse { response: Some(Error(e.to_proto())) });
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, MSG_ID_ERR_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  let products = c.store.hero_products(ctx.clone()).await;

  match products {
    Ok(products) => {
      return Ok(Response::new(HeroProductsResponse {
        response: Some(Data(HeroProductsResponseData {
          category_slider: Some(products.category_slider.unwrap()),
          welcome_deals_slider: Some(products.welcome_deals_slider.unwrap()),
        })),
      }))
    }
    Err(err) => {
      return Ok(return_err(ie(Box::new(err))));
    }
  }
}
