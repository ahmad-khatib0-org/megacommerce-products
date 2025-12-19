use std::{collections::HashMap, str::FromStr, sync::Arc};

use megacommerce_proto::{
  product_create_response::Response::{Data as ResData, Error as ResError},
  ProductCreateRequest, ProductCreateResponse, ProductMedia, ProductMediaImage,
  ProductMediaVariant, ProductMediaVideo, SuccessResponseData,
};
use megacommerce_shared::models::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, MSG_ID_ERR_INTERNAL},
  images::ImageValidationResult,
  r_lock::RLock,
  translate::tr,
};
use tokio::spawn;
use tonic::Code;
use tonic::{Request, Response, Status};
use ulid::Ulid;

use crate::{
  controller::{audit::process_audit, Controller},
  models::{
    audit::{AuditRecord, EventName::ProductCreate, EventParameterKey, EventStatus::Fail},
    product_create::{
      products_create_auditable_v1, products_create_is_valid, products_create_pre_save,
    },
  },
  server::object_storage::ObjectStorage,
};

pub(super) async fn product_create(
  c: &Controller,
  req: Request<ProductCreateRequest>,
) -> Result<Response<ProductCreateResponse>, Status> {
  let start = std::time::Instant::now();
  c.metrics.product_create_total.inc();
  
  let path = "products.controller.product_create";
  let cfg = c.cfg.get().await;
  let ctx = req.extensions().get::<Arc<Context>>().cloned().unwrap();
  let lang = ctx.accept_language();

  let mut audit = AuditRecord::new(ctx.clone(), ProductCreate, Fail);
  let pro = req.into_inner();

  let return_err = |e: AppError| {
    c.metrics.record_product_create_error();
    Response::new(ProductCreateResponse { response: Some(ResError(e.to_proto()))})
  };

  let pro_clone = pro.clone();
  let audit_data_future = spawn(async move { products_create_auditable_v1(&pro_clone) });

  let identity = pro.identity.clone().unwrap_or_default();
  let sub = c.cache.subcategory_data(&identity.category, &identity.subcategory, lang);

  let is_valid = products_create_is_valid(ctx.clone(), &pro, sub, &cfg);
  if is_valid.is_err() {
    return Ok(return_err(is_valid.unwrap_err()));
  }

  let pro_db = products_create_pre_save(ctx.clone(), &pro);
  if pro_db.is_err() {
    return Ok(return_err(pro_db.unwrap_err().to_internal(ctx.clone(), path.into())));
  }

  let is_valid = is_valid.unwrap();
  let pro_db = &mut pro_db.unwrap();

  let media_upload = upload_media(
    ctx.clone(),
    &c.storage,
    is_valid.media_validation_results_with_variants,
    is_valid.media_validation_results_no_variants,
    &pro_db.variants_ids,
    &pro_db.main_variant_key,
  )
  .await;
  if media_upload.is_err() {
    // TODO: consider removing the inserted product and all its related data
    return Ok(return_err(media_upload.unwrap_err()));
  }

  pro_db.product.media = Some(media_upload.unwrap());
  if let Err(err) = c.store.product_create(ctx.clone(), &pro_db.product).await {
    return Ok(return_err(err.to_app_error_internal(ctx.clone(), path.into())));
  }

  let audit_data = audit_data_future.await.unwrap_or_default();
  audit.set_event_parameter(EventParameterKey::ProductCreate, audit_data);
  audit.success();
  spawn(async move {
    process_audit(&audit);
  });

  let message = tr::<()>(lang, "products.create.successfully", None)
    .unwrap_or("The Product created successfully!".to_string());

  let duration = start.elapsed().as_secs_f64();
  c.metrics.record_product_create_success(duration);

  Ok(Response::new(ProductCreateResponse {
    response: Some(ResData(SuccessResponseData { message: Some(message), ..Default::default() })),
  }))
}

// TODO:
// consider caching the uploading status in redis
// consider uploading the videos
async fn upload_media(
  ctx: Arc<Context>,
  uploader: &RLock<ObjectStorage>,
  variants: HashMap<String, HashMap<String, ImageValidationResult>>,
  no_variants: HashMap<String, ImageValidationResult>,
  variants_ids: &HashMap<String, String>,
  main_variant_key: &String,
) -> Result<ProductMedia, AppError> {
  let mut media = ProductMedia { media: HashMap::new() };

  let uploader = uploader.get().await;
  let path = "products.controller.upload_media".to_string();
  let ie = |err: BoxedErr, msg: &str| {
    AppError::new(
      ctx.clone(),
      path.clone(),
      MSG_ID_ERR_INTERNAL,
      None,
      msg,
      Code::Internal.into(),
      Some(AppErrorErrors { err: Some(err), ..Default::default() }),
    )
  };

  if variants.len() > 0 {
    for variant in variants.iter() {
      let db_var_id =
        variants_ids.get(variant.0).expect(&format!("{}: the variant id is not found!", path));

      let mut images: HashMap<String, ProductMediaImage> = HashMap::new();
      let videos: HashMap<String, ProductMediaVideo> = HashMap::new();
      for (_, result) in variant.1.iter() {
        let id = Ulid::new().to_string();
        let image_id = Ulid::new().to_string();
        let _ = uploader
          .upload_file(&id, result.decoded_data.clone(), result.format.to_mime_type())
          .await
          .map_err(|err| ie(err, "failed to update an image"))?;
        images.insert(
          image_id,
          ProductMediaImage {
            url: id,
            format: String::from_str(result.format.to_mime_type()).unwrap(),
            size: result.size_bytes as u64,
          },
        );
      }
      media.media.insert(db_var_id.to_string(), ProductMediaVariant { images, videos });
    }
  } else {
    let mut images: HashMap<String, ProductMediaImage> = HashMap::new();
    let videos: HashMap<String, ProductMediaVideo> = HashMap::new();
    for (_, result) in no_variants.iter() {
      let id = Ulid::new().to_string();
      let _ = uploader
        .upload_file(&id, result.decoded_data.clone(), result.format.to_mime_type())
        .await
        .map_err(|err| ie(err, "failed to update an image"))?;
      images.insert(
        Ulid::new().to_string(),
        ProductMediaImage {
          format: String::from_str(result.format.to_mime_type()).unwrap(),
          url: id,
          size: result.size_bytes as u64,
        },
      );
    }
    let db_var_id = variants_ids
      .get(main_variant_key)
      .expect(&format!("{}: the main variant id is not found!", path));
    media.media.insert(db_var_id.to_string(), ProductMediaVariant { images, videos });
  }

  Ok(media)
}
