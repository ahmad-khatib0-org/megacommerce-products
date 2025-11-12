use std::{collections::HashMap, sync::Arc};

use chrono::NaiveDate;
use image::ImageFormat;
use lazy_static::lazy_static;
use megacommerce_proto::{
  product_create_request_details::Details::{
    WithVariants as DetailsVariants, WithoutVariants as DetailsNoVariants,
  },
  product_create_request_media::Media::{
    WithVariants as MediaWithVariants, WithoutVariants as MediaNoVariants,
  },
  product_create_request_offer::Pricing::{
    WithVariants as OfferWithVariants, WithoutVariants as OfferNoVariants,
  },
  validation_field::Rule,
  Any, Attachment, Attachments, Config, ConfigProducts, NumericRuleType, Product,
  ProductBulletPoint, ProductCreateRequest, ProductCreateRequestDescription,
  ProductCreateRequestDetails, ProductCreateRequestDetailsWithVariants,
  ProductCreateRequestDetailsWithoutVariants, ProductCreateRequestIdentity,
  ProductCreateRequestMedia, ProductCreateRequestMediaWithVariants,
  ProductCreateRequestMediaWithoutVariants, ProductCreateRequestOffer,
  ProductCreateRequestOfferMinimumOrder, ProductCreateRequestOfferWithVariants,
  ProductCreateRequestOfferWithoutVariants, ProductCreateRequestSafety,
  ProductDataResponseSubcategory, ProductDetails, ProductDetailsVariant, ProductOffer,
  ProductOfferMinimumOrder, ProductOfferVariant, ProductSafety, StringRuleType, Subcategory,
  ValidationField, ValidationFieldNumeric, ValidationFieldRegex, ValidationFieldString,
};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{AppError, AppErrorError, AppErrorErrors},
    files::UnitSizeType,
    images::{
      validate_base64_image, ImageValidationConfig, ImageValidationError, ImageValidationResult,
    },
    products::SubcategoryAttributeType,
  },
  utils::{
    grpc::{grpc_deserialize_any, AnyExt, AnyValue},
    time::time_get_millis,
  },
};
use serde_json::{json, Value};
use tonic::Code;
use ulid::Ulid;

use crate::{
  data::currencies::CURRENCY_LIST,
  models::products::{
    product_id_is_validate, ProductCreateStepsNames, ProductFulfillmentType,
    ProductOfferingCondition, PRODUCT_BRAND_NAME_MAX_LENGTH, PRODUCT_BRAND_NAME_MIN_LENGTH,
    PRODUCT_DESCRIPTION_BULLET_POINTS_MAX_LENGTH, PRODUCT_DESCRIPTION_BULLET_POINTS_MIN_LENGTH,
    PRODUCT_DESCRIPTION_BULLET_POINT_MAX_LENGTH, PRODUCT_DESCRIPTION_BULLET_POINT_MIN_LENGTH,
    PRODUCT_DESCRIPTION_MAX_LENGTH, PRODUCT_DESCRIPTION_MIN_LENGTH, PRODUCT_ID_TYPES,
    PRODUCT_MINIMUM_INVENTORY_QUANTITY, PRODUCT_MINIMUM_ORDER_MAX_OPTIONS,
    PRODUCT_MINIMUM_ORDER_MIN_OPTIONS, PRODUCT_OFFERING_CONDITION_NOTE_MAX_LENGTH,
    PRODUCT_OFFERING_CONDITION_NOTE_MIN_LENGTH, PRODUCT_SKU_MAX_LENGTH, PRODUCT_SKU_MIN_LENGTH,
    PRODUCT_TITLE_MAX_LENGTH, PRODUCT_TITLE_MIN_LENGTH, PRODUCT_VARIATION_TITLE_MAX_LENGTH,
    PRODUCT_VARIATION_TITLE_MIN_LENGTH,
  },
  utils::slug::Slug,
};

use super::products::ProductStatus;

struct ProductCreateOfferPricingSharedFields {
  pub sku: String,
  pub quantity: u64,
  pub price: String,
  pub offering_condition: String,
  pub condition_note: Option<String>,
  pub list_price: Option<String>,
  pub has_sale_price: Option<bool>,
  pub sale_price: Option<String>,
  pub sale_price_start: Option<String>,
  pub sale_price_end: Option<String>,
  pub has_minimum_orders: bool,
  pub minimum_orders: Vec<ProductCreateRequestOfferMinimumOrder>,
}

lazy_static! {
  pub static ref ERR_REQUIRED: AppErrorError =
    AppErrorError { id: "required".to_string(), params: None };
  pub static ref ERR_INVALID_NUM: AppErrorError =
    AppErrorError { id: "form.fields.invalid_number".to_string(), params: None };
  pub static ref ERR_INVALID_INP: AppErrorError =
    AppErrorError { id: "form.field.invalid_input".to_string(), params: None };
  pub static ref ERR_INVALID_DATE: AppErrorError =
    AppErrorError { id: "form.fields.invalid_date".to_string(), params: None };
  pub static ref ERR_GT_0: AppErrorError =
    AppErrorError { id: "form.fields.bigger_than_zero".into(), params: None };
  pub static ref ERR_MISSIN_FID: AppErrorError =
    AppErrorError { id: "form.field.id.missing_or_invalid".into(), params: None };
  pub static ref ERR_MUST_CHECK: AppErrorError =
    AppErrorError { id: "form.field.checkbox.checked.error".into(), params: None };
}

#[derive(Debug)]
pub struct ProductCreateIsValidResult {
  pub media_validation_results_no_variants: HashMap<String, ImageValidationResult>,
  pub media_validation_results_with_variants:
    HashMap<String, HashMap<String, ImageValidationResult>>,
}

impl Default for ProductCreateIsValidResult {
  fn default() -> Self {
    Self {
      media_validation_results_no_variants: HashMap::new(),
      media_validation_results_with_variants: HashMap::new(),
    }
  }
}

pub fn products_create_is_valid(
  ctx: Arc<Context>,
  product: &ProductCreateRequest,
  subcategory_data: Option<ProductDataResponseSubcategory>,
  config: &Config,
) -> Result<ProductCreateIsValidResult, AppError> {
  let mut result = ProductCreateIsValidResult { ..Default::default() };

  let identity = product.identity.clone().unwrap_or_default();
  let description = product.description.clone().unwrap_or_default();
  let details = product.details.clone().unwrap_or_default();
  let media = product.media.clone().unwrap_or_default();
  let offer = product.offer.clone().unwrap_or_default();
  let safety = product.safety.clone().unwrap_or_default();

  let mut errors: HashMap<String, AppErrorError> = HashMap::new();

  identity_form_validation(identity, &mut errors, &subcategory_data);
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  description_form_validation(description, &mut errors);
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  let sub_data = subcategory_data.unwrap();
  let sub = sub_data.data.unwrap();

  details_form_validation(details, &mut errors, &sub);
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  let media_validaiton = media_form_validation(media, &mut errors, config);
  result.media_validation_results_with_variants = media_validaiton.0;
  result.media_validation_results_no_variants = media_validaiton.1;
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  offer_form_validation(offer, &mut errors);
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  validate_safety_form(safety, &mut errors, &sub);
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  Ok(result)
}

fn identity_form_validation(
  identity: ProductCreateRequestIdentity,
  errors: &mut HashMap<String, AppErrorError>,
  subcategory_data: &Option<ProductDataResponseSubcategory>,
) {
  let title_len = identity.title.chars().count();
  if title_len < PRODUCT_TITLE_MIN_LENGTH || title_len > PRODUCT_TITLE_MAX_LENGTH {
    let params = Some(HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_TITLE_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_TITLE_MAX_LENGTH.into())),
    ]));
    errors
      .insert("identity.title".into(), AppErrorError { id: "products.title.error".into(), params });
  }
  if identity.category.is_empty() {
    errors.insert(
      "identity.category".into(),
      AppErrorError { id: "products.category.missing.error".into(), params: None },
    );
  }
  if identity.subcategory.is_empty() {
    errors.insert(
      "identity.subcategory".into(),
      AppErrorError { id: "products.subcategory.missing.error".into(), params: None },
    );
  }
  if subcategory_data.is_none() {
    errors.insert(
      "identity.subcategory".into(),
      AppErrorError { id: "products.type.required".into(), params: None },
    );
  }

  let brand_len = identity.brand_name.chars().count();
  if !identity.no_brand
    && (brand_len > PRODUCT_BRAND_NAME_MAX_LENGTH || brand_len < PRODUCT_BRAND_NAME_MIN_LENGTH)
  {
    let params = Some(HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_BRAND_NAME_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_BRAND_NAME_MAX_LENGTH.into())),
    ]));
    errors.insert(
      "identity.brand_name".into(),
      AppErrorError { id: "products.brand_name.error".into(), params },
    );
  }
  if !identity.no_product_id && identity.product_id.is_empty() {
    errors.insert(
      "identity.product_id".into(),
      AppErrorError { id: "products.external_product_id.required".into(), params: None },
    );
  }
  if !identity.no_product_id && identity.product_id_type.is_empty() {
    errors.insert(
      "identity.product_id_type".into(),
      AppErrorError { id: "products.external_product_id.required".into(), params: None },
    );
  }
  if !identity.no_product_id && !PRODUCT_ID_TYPES.contains(&identity.product_id_type.as_str()) {
    errors.insert(
      "identity.product_id_type".into(),
      AppErrorError { id: "products.external_product_id_type.invalid".into(), params: None },
    );
  }
  if !identity.no_product_id
    && !product_id_is_validate(&identity.product_id_type, &identity.product_id)
  {
    errors.insert(
      "identity.product_id".into(),
      AppErrorError { id: "products.external_product_id.invalid".into(), params: None },
    );
  }
}

fn description_form_validation(
  description: ProductCreateRequestDescription,
  errors: &mut HashMap<String, AppErrorError>,
) {
  let desc_len = description.description.chars().count();
  if desc_len < PRODUCT_DESCRIPTION_MIN_LENGTH || desc_len > PRODUCT_DESCRIPTION_MAX_LENGTH {
    let params = Some(HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_DESCRIPTION_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_DESCRIPTION_MAX_LENGTH.into())),
    ]));
    errors.insert(
      "description.description".into(),
      AppErrorError { id: "products.description.error".into(), params },
    );
  }
  let bullet_points_len = description.bullet_points.len();
  if bullet_points_len < PRODUCT_DESCRIPTION_BULLET_POINTS_MIN_LENGTH
    || bullet_points_len > PRODUCT_DESCRIPTION_BULLET_POINTS_MAX_LENGTH
  {
    let params = Some(HashMap::from([
      ("Min".to_string(), Value::Number(PRODUCT_DESCRIPTION_BULLET_POINTS_MIN_LENGTH.into())),
      ("Max".to_string(), Value::Number(PRODUCT_DESCRIPTION_BULLET_POINTS_MAX_LENGTH.into())),
    ]));
    errors.insert(
      "description.bullet_points.count".into(),
      AppErrorError { id: "products.bullet_points.count.error".into(), params },
    );
  }
  for bp in description.bullet_points.iter() {
    let len = bp.bullet_point.len();
    if len < PRODUCT_DESCRIPTION_BULLET_POINT_MIN_LENGTH
      || len > PRODUCT_DESCRIPTION_BULLET_POINT_MAX_LENGTH
    {
      let params = Some(HashMap::from([
        ("Min".to_string(), Value::Number(PRODUCT_DESCRIPTION_BULLET_POINTS_MIN_LENGTH.into())),
        ("Max".to_string(), Value::Number(PRODUCT_DESCRIPTION_BULLET_POINTS_MAX_LENGTH.into())),
      ]));
      errors.insert(
        format!("description.bullet_points.{}.bullet_point", bp.id),
        AppErrorError { id: "products.bullet_point.length.error".into(), params },
      );
    }
  }
}

fn details_form_validation(
  details: ProductCreateRequestDetails,
  errors: &mut HashMap<String, AppErrorError>,
  sub: &Subcategory,
) {
  let step = ProductCreateStepsNames::Details;
  let default_validation = &ValidationField::default();

  if details.details.is_none() {
    errors.insert(
      missing_form(&ProductCreateStepsNames::Details),
      AppErrorError { id: "products.details.form.missing".into(), params: None },
    );
    return;
  }
  match details.details.unwrap() {
    DetailsVariants(varinats) => {
      details_with_variations_form_validation(&varinats, errors, sub);
    }
    DetailsNoVariants(form) => details_without_variations_form_validation(form, errors, sub),
  };

  for (field_name, field_value) in details.shared.iter() {
    let found_field = sub.attributes.get(field_name);
    if found_field.is_none() {
      unknown_field(errors, &step, None, &field_name);
      break;
    }

    let field = found_field.unwrap();
    validate_attribute(
      field_value,
      field_name,
      field.r#type.as_ref(),
      errors,
      &step,
      None,
      field.validation.as_ref().unwrap_or(default_validation),
      field.required,
      field.string_array.as_ref(),
    );
  }
}

// TODO: check for the fields that are required in &Subcategory, but user didn't send them
fn details_with_variations_form_validation(
  variants: &ProductCreateRequestDetailsWithVariants,
  errors: &mut HashMap<String, AppErrorError>,
  sub: &Subcategory,
) {
  let step = ProductCreateStepsNames::Details;
  let default_validation = &ValidationField::default();

  for variant in variants.variants.iter() {
    let form_id = grpc_deserialize_any(variant.form.get("id").unwrap_or(&Any::default()));
    let id = match form_id {
      AnyValue::String(form_id) => form_id,
      _ => "".to_string(),
    };
    if id.is_empty() {
      errors.insert(missing_form_id(&step), ERR_MISSIN_FID.clone());
      break;
    }

    let form_title = grpc_deserialize_any(variant.form.get("title").unwrap_or(&Any::default()));
    let title = match form_title {
      AnyValue::String(t) => t,
      _ => "".to_string(),
    };
    if title.is_empty() {
      let err = AppErrorError { id: "products.variations.title.missing".to_string(), params: None };
      field_error(errors, &step, Some(&id), "title", err);
      break;
    }
    if title.len() < PRODUCT_VARIATION_TITLE_MIN_LENGTH
      || title.len() > PRODUCT_VARIATION_TITLE_MAX_LENGTH
    {
      let params = Some(HashMap::from([
        ("Min".to_string(), Value::Number(PRODUCT_VARIATION_TITLE_MIN_LENGTH.into())),
        ("Max".to_string(), Value::Number(PRODUCT_VARIATION_TITLE_MAX_LENGTH.into())),
      ]));
      let err = AppErrorError { id: "products.variations.title.error".to_string(), params };
      field_error(errors, &step, Some(&id), "title", err);
      break;
    }

    for (field_name, field_value) in variant.form.iter() {
      if field_name.as_str() == "id" || field_name.as_str() == "title" {
        continue; // this field got validated
      }

      let field = sub.attributes.get(field_name);
      if field.is_none() {
        unknown_field(errors, &step, Some(&id), &field_name);
        break;
      }

      let field = field.unwrap();
      if !field.include_in_variants {
        let err = AppErrorError { id: "form.field.not_customizable".into(), params: None };
        field_error(errors, &step, Some(&id), field_name, err);
        continue;
      }

      validate_attribute(
        field_value,
        field_name,
        field.r#type.as_ref(),
        errors,
        &step,
        Some(&id),
        field.validation.as_ref().unwrap_or(default_validation),
        field.required,
        field.string_array.as_ref(),
      );
    }
  }
}

fn details_without_variations_form_validation(
  form: ProductCreateRequestDetailsWithoutVariants,
  errors: &mut HashMap<String, AppErrorError>,
  sub: &Subcategory,
) {
  let step = ProductCreateStepsNames::Details;
  let default_validation = &ValidationField::default();

  for (field_name, field_value) in form.form.iter() {
    let found_field = sub.attributes.get(field_name);
    if found_field.is_none() {
      unknown_field(errors, &step, None, &field_name);
      break;
    }

    let field = found_field.unwrap();
    validate_attribute(
      field_value,
      field_name,
      field.r#type.as_ref(),
      errors,
      &step,
      None,
      field.validation.as_ref().unwrap_or(default_validation),
      field.required,
      field.string_array.as_ref(),
    );
  }
}

// TODO:
// handle resumeable uploading case (for big media files)
// handle validating videos also
// handle validating checksum of attachments
fn media_form_validation(
  media: ProductCreateRequestMedia,
  errors: &mut HashMap<String, AppErrorError>,
  config: &Config,
) -> (HashMap<String, HashMap<String, ImageValidationResult>>, HashMap<String, ImageValidationResult>)
{
  let mut validation_results_variants: HashMap<String, HashMap<String, ImageValidationResult>> =
    HashMap::new();
  let mut validation_results_no_variants: HashMap<String, ImageValidationResult> = HashMap::new();

  if media.media.is_none() {
    errors.insert(
      missing_form(&ProductCreateStepsNames::Details),
      AppErrorError { id: "products.media.form.missing".into(), params: None },
    );
    return (validation_results_variants, validation_results_no_variants);
  }

  let form = media.media.unwrap();
  let cfg = config.products.as_ref().unwrap();
  let allowed_formats = || -> Vec<ImageFormat> {
    let mut allowed: Vec<ImageFormat> = vec![];
    for format in &cfg.product_image_accepted_formats {
      if format == "image/png" {
        allowed.push(ImageFormat::Png);
      } else if format == "image/webp" {
        allowed.push(ImageFormat::WebP);
      } else if format == "image/jpeg" {
        allowed.push(ImageFormat::Jpeg);
      }
    }
    allowed
  };

  let img_config = &ImageValidationConfig {
    max_size_bytes: (cfg.product_image_max_size_mb * 1024 * 1024) as usize,
    allowed_formats: allowed_formats(),
    max_width: cfg.product_image_max_width as u32,
    max_height: cfg.product_image_max_height as u32,
    min_width: cfg.product_image_min_width as u32,
    min_height: cfg.product_image_min_height as u32,
  };
  match form {
    MediaWithVariants(m) => {
      validation_results_variants =
        media_with_variations_form_validation(m, errors, config, img_config);
    }
    MediaNoVariants(m) => {
      validation_results_no_variants =
        media_without_variations_form_validation(m, errors, config, img_config);
    }
  }

  (validation_results_variants, validation_results_no_variants)
}

fn media_with_variations_form_validation(
  forms: ProductCreateRequestMediaWithVariants,
  errors: &mut HashMap<String, AppErrorError>,
  config: &Config,
  img_cfg: &ImageValidationConfig,
) -> HashMap<String, HashMap<String, ImageValidationResult>> {
  let step = &ProductCreateStepsNames::Media;
  let cfg = config.products.as_ref().unwrap();
  let min_count = cfg.product_images_min_count_per_variant.clone() as usize;
  let max_count = cfg.product_images_max_count_per_variant.clone() as usize;
  let mut validation_results: HashMap<String, HashMap<String, ImageValidationResult>> =
    HashMap::new();

  if forms.images.is_empty() {
    let err = AppErrorError { id: "products.media.missing_images".into(), params: None };
    field_error(errors, step, None, "count", err);
  }

  for form in forms.images.iter() {
    if form.1.attachments.len() < min_count || form.1.attachments.len() > max_count {
      let params = Some(HashMap::from([
        ("Min".into(), Value::Number(min_count.into())),
        ("Max".into(), Value::Number(max_count.into())),
      ]));
      let err = AppErrorError { id: "products.media.variant_images.count".into(), params };
      field_error(errors, step, Some(form.0), "count", err);
      return validation_results;
    }

    if form.0.is_empty() {
      errors.insert(missing_form_id(step), ERR_MISSIN_FID.clone());
      return validation_results;
    }
    for attachment in form.1.attachments.iter() {
      let validation_result = validate_image(cfg, img_cfg, errors, step, Some(form.0), attachment);
      if validation_result.is_some() {
        validation_results
          .entry(form.0.clone())
          .or_insert_with(HashMap::new)
          .insert(attachment.id.clone(), validation_result.unwrap());
      }
    }
  }

  validation_results
}

fn media_without_variations_form_validation(
  form: ProductCreateRequestMediaWithoutVariants,
  errors: &mut HashMap<String, AppErrorError>,
  config: &Config,
  img_cfg: &ImageValidationConfig,
) -> HashMap<String, ImageValidationResult> {
  let step = &ProductCreateStepsNames::Media;
  let cfg = config.products.as_ref().unwrap();
  let min_count = cfg.product_images_min_count_per_variant.clone() as usize;
  let max_count = cfg.product_images_max_count_per_variant.clone() as usize;
  let mut validation_results: HashMap<String, ImageValidationResult> = HashMap::new();

  if form.images.is_empty() {
    let err = AppErrorError { id: "products.media.missing_images".into(), params: None };
    field_error(errors, step, None, "count", err);
  }

  if form.images.len() < min_count || form.images.len() > max_count {
    let params = Some(HashMap::from([
      ("Min".into(), Value::Number(min_count.into())),
      ("Max".into(), Value::Number(max_count.into())),
    ]));
    let err = AppErrorError { id: "products.media.variant_images.count".into(), params };
    field_error(errors, step, None, "count", err);
    return validation_results;
  }

  for attachment in form.images.iter() {
    let validation_result = validate_image(cfg, img_cfg, errors, step, None, attachment);
    if validation_result.is_some() {
      validation_results.insert(attachment.id.clone(), validation_result.unwrap());
    }
  }

  validation_results
}

fn validate_image(
  cfg: &ConfigProducts,
  img_cfg: &ImageValidationConfig,
  errors: &mut HashMap<String, AppErrorError>,
  step: &ProductCreateStepsNames,
  form_id: Option<&str>,
  attachment: &Attachment,
) -> Option<ImageValidationResult> {
  let max_size = cfg.product_image_max_size_mb;
  let min_w_dim = cfg.product_image_min_width;
  let max_w_dim = cfg.product_image_max_width;
  let min_h_dim = cfg.product_image_min_height;
  let max_h_dim = cfg.product_image_max_height;
  let image_types = cfg.product_image_accepted_formats.join(", ");
  let img_data = attachment.base64.as_str();

  let result = validate_base64_image(&img_data, img_cfg);
  if result.is_err() {
    match result.as_ref().unwrap_err() {
      ImageValidationError::LargeImage(_) => {
        let params = Some(HashMap::from([
          ("Max".into(), Value::Number(max_size.into())),
          ("Unit".into(), Value::String(UnitSizeType::MB.as_str().to_string())),
        ]));
        let err = AppErrorError { id: "image.max_size.error".to_string(), params };
        field_error(errors, step, form_id, &attachment.id, err);
      }
      ImageValidationError::InvalidBase64(_) => {
        let err = AppErrorError { id: "image.data.invalid".to_string(), params: None };
        field_error(errors, step, form_id, &attachment.id, err);
      }
      ImageValidationError::UnknownFormat(_) => {
        let err = AppErrorError { id: "image.type.unknown".to_string(), params: None };
        field_error(errors, step, form_id, &attachment.id, err);
      }
      ImageValidationError::NotAllowedFormat(_) => {
        let params = Some(HashMap::from([("Types".into(), Value::String(image_types.clone()))]));
        let err = AppErrorError { id: "image.type.unsupported".to_string(), params };
        field_error(errors, step, form_id, &attachment.id, err);
      }
      ImageValidationError::SmallDimensions(_) => {
        let params = Some(HashMap::from([(
          "Dimensions".into(),
          Value::String(format!("{}-{}", min_w_dim, min_h_dim)),
        )]));
        let err = AppErrorError { id: "image.dimensions.min.error".to_string(), params };
        field_error(errors, step, form_id, &attachment.id, err);
      }
      ImageValidationError::LargeDimensions(_) => {
        let params = Some(HashMap::from([(
          "Dimensions".into(),
          Value::String(format!("{}-{}", max_w_dim, max_h_dim)),
        )]));
        let err = AppErrorError { id: "image.dimensions.max.error".to_string(), params };
        field_error(errors, step, form_id, &attachment.id, err);
      }
      ImageValidationError::UnknownDimensions(_) => {
        let err = AppErrorError { id: "image.dimensions.unknown.error".to_string(), params: None };
        field_error(errors, step, form_id, &attachment.id, err);
      }
    }
    return None;
  }

  Some(result.unwrap())
}

fn offer_form_validation(
  form: ProductCreateRequestOffer,
  errors: &mut HashMap<String, AppErrorError>,
) {
  let step = &ProductCreateStepsNames::Offer;
  if form.pricing.is_none() {
    errors.insert(
      missing_form(&ProductCreateStepsNames::Offer),
      AppErrorError { id: "products.offer.form.missing".into(), params: None },
    );
    return;
  }

  match form.pricing.unwrap() {
    OfferWithVariants(offer) => offer_with_variations_form_validation(offer, errors),
    OfferNoVariants(offer) => offer_without_variations_form_validation(offer, errors),
  }

  if CURRENCY_LIST.iter().find(|cur| cur.code == form.currency).is_none() {
    let err = AppErrorError { id: "products.currency_code.error".to_string(), params: None };
    field_error(errors, step, None, "currency", err);
  }
  if !ProductFulfillmentType::as_slice().contains(&form.fulfillment_type.as_str()) {
    field_error(errors, step, None, "fulfillment_type", ERR_INVALID_INP.clone());
  }
  if form.processing_time <= 0 {
    field_error(errors, step, None, "processing_time", ERR_GT_0.clone());
  }
}

fn offer_with_variations_form_validation(
  form: ProductCreateRequestOfferWithVariants,
  errors: &mut HashMap<String, AppErrorError>,
) {
  let step = &ProductCreateStepsNames::Offer;
  for form in form.variants.iter() {
    if form.id.is_empty() {
      errors.insert(missing_form_id(step), ERR_MISSIN_FID.clone());
      continue;
    }
    let offer = &ProductCreateOfferPricingSharedFields {
      sku: form.sku.clone(),
      quantity: form.quantity,
      price: form.price.clone(),
      offering_condition: form.offering_condition.clone(),
      condition_note: form.condition_note.clone(),
      list_price: form.list_price.clone(),
      has_sale_price: form.has_sale_price,
      sale_price: form.sale_price.clone(),
      sale_price_start: form.sale_price_start.clone(),
      sale_price_end: form.sale_price_end.clone(),
      has_minimum_orders: form.has_minimum_orders,
      minimum_orders: form.minimum_orders.clone(),
    };
    validate_offer_pricing(offer, step, Some(&form.id), errors);
  }
}

fn offer_without_variations_form_validation(
  form: ProductCreateRequestOfferWithoutVariants,
  errors: &mut HashMap<String, AppErrorError>,
) {
  let step = &ProductCreateStepsNames::Offer;
  let offer = &ProductCreateOfferPricingSharedFields {
    sku: form.sku,
    quantity: form.quantity,
    price: form.price,
    offering_condition: form.offering_condition,
    condition_note: form.condition_note,
    list_price: form.list_price,
    has_sale_price: form.has_sale_price,
    sale_price: form.sale_price,
    sale_price_start: form.sale_price_start,
    sale_price_end: form.sale_price_end,
    has_minimum_orders: form.has_minimum_orders,
    minimum_orders: form.minimum_orders,
  };
  validate_offer_pricing(offer, step, None, errors);
}

fn validate_offer_pricing(
  offer: &ProductCreateOfferPricingSharedFields,
  step: &ProductCreateStepsNames,
  form_id: Option<&str>,
  errors: &mut HashMap<String, AppErrorError>,
) {
  if offer.quantity < PRODUCT_MINIMUM_INVENTORY_QUANTITY {
    let params = Some(HashMap::from([(
      "Min".into(),
      Value::Number(PRODUCT_MINIMUM_INVENTORY_QUANTITY.into()),
    )]));
    let err = AppErrorError { id: "form.field.min".into(), params };
    field_error(errors, step, form_id, "quantity", err);
  }

  let sku_len = offer.sku.len();
  if sku_len < PRODUCT_SKU_MIN_LENGTH || sku_len > PRODUCT_SKU_MAX_LENGTH {
    let params = Some(HashMap::from([
      ("Min".into(), Value::Number(PRODUCT_SKU_MIN_LENGTH.into())),
      ("Max".into(), Value::Number(PRODUCT_SKU_MAX_LENGTH.into())),
    ]));
    let err = AppErrorError { id: "products.sku.error".into(), params };
    field_error(errors, step, form_id, "sku", err);
  }

  let price = offer.price.parse::<f64>();
  match price {
    Ok(price) => {
      if price <= 0.0 {
        field_error(errors, step, form_id, "price", ERR_GT_0.clone());
      }
    }
    Err(_) => {
      field_error(errors, step, form_id, "price", ERR_INVALID_NUM.clone());
    }
  }

  if !ProductOfferingCondition::as_slice().contains(&offer.offering_condition.as_str()) {
    field_error(errors, step, form_id, "offering_condition", ERR_INVALID_INP.clone());
  }

  match ProductOfferingCondition::from_str(&offer.offering_condition) {
    ProductOfferingCondition::Used => {
      let cond_len = offer.condition_note.clone().unwrap_or_default().len();
      if cond_len < PRODUCT_OFFERING_CONDITION_NOTE_MIN_LENGTH
        || cond_len > PRODUCT_OFFERING_CONDITION_NOTE_MAX_LENGTH
      {
        let params = Some(HashMap::from([
          ("Min".into(), Value::Number(PRODUCT_OFFERING_CONDITION_NOTE_MIN_LENGTH.into())),
          ("Max".into(), Value::Number(PRODUCT_OFFERING_CONDITION_NOTE_MAX_LENGTH.into())),
        ]));
        let err = AppErrorError { id: "products.condition_note.error".into(), params };
        field_error(errors, step, form_id, "condition_note", err);
      }
    }
    _ => {}
  }

  let ls_price = offer.list_price.clone().unwrap_or_default();
  if !ls_price.is_empty() {
    match ls_price.parse::<f64>() {
      Ok(ls_price) => {
        if ls_price <= price.clone().unwrap_or_default() {
          let err = AppErrorError { id: "products.list_price.error".into(), params: None };
          field_error(errors, step, form_id, "list_price", err);
        }
      }
      Err(_) => {
        field_error(errors, step, form_id, "list_price", ERR_INVALID_NUM.clone());
      }
    }
  }

  if offer.has_sale_price.unwrap_or_default() {
    let sale_price = offer.sale_price.clone().unwrap_or_default();
    if sale_price.is_empty() {
      field_error(errors, step, form_id, "sale_price", ERR_REQUIRED.clone());
    } else {
      match sale_price.parse::<f64>() {
        Ok(sp) => {
          if sp <= price.unwrap_or_default() {
            let err = AppErrorError { id: "products.sale_price.lesser".into(), params: None };
            field_error(errors, step, form_id, "sale_price", err);
          }
        }
        Err(_) => {
          field_error(errors, step, form_id, "sale_price", ERR_INVALID_NUM.clone());
        }
      }
    }

    let start = offer.sale_price_start.clone().unwrap_or_default();
    let end = offer.sale_price_end.clone().unwrap_or_default();
    if start.is_empty() {
      field_error(errors, step, form_id, "sale_price_start", ERR_REQUIRED.clone());
    } else {
      match NaiveDate::parse_from_str(&start, "%Y-%m-%d") {
        Ok(start_date) => {
          if !end.is_empty() {
            match NaiveDate::parse_from_str(&end, "%Y-%m-%d") {
              Ok(end_date) => {
                if end_date <= start_date {
                  let err =
                    AppErrorError { id: "products.sale_price_end.lesser".into(), params: None };
                  field_error(errors, step, form_id, "sale_price_end", err);
                }
              }
              Err(_) => {
                field_error(errors, step, form_id, "sale_price_end", ERR_INVALID_DATE.clone());
              }
            }
          }
        }
        Err(_) => {
          field_error(errors, step, form_id, "sale_price_start", ERR_INVALID_DATE.clone());
        }
      }
    }
    if end.is_empty() {
      field_error(errors, step, form_id, "sale_price_end", ERR_REQUIRED.clone());
    }
  }

  if offer.has_minimum_orders {
    let mo_len = offer.minimum_orders.len();
    if mo_len < PRODUCT_MINIMUM_ORDER_MIN_OPTIONS || mo_len > PRODUCT_MINIMUM_ORDER_MAX_OPTIONS {
      let params = Some(HashMap::from([
        ("Min".into(), Value::Number(PRODUCT_MINIMUM_ORDER_MIN_OPTIONS.into())),
        ("Max".into(), Value::Number(PRODUCT_MINIMUM_ORDER_MAX_OPTIONS.into())),
      ]));
      let err = AppErrorError { id: "products.minimum_order_options.count.error".into(), params };
      field_error(errors, step, form_id, "minimum_orders.count", err);
    }

    for mo in offer.minimum_orders.iter() {
      if mo.id.is_empty() {
        let key = match form_id {
          Some(fid) => format!("{}.{}.minimum_orders.form_id.missing", step.as_str(), fid),
          None => format!("{}.minimum_orders.form_id.missing", step.as_str()),
        };
        errors.insert(key, ERR_MISSIN_FID.clone());
        continue;
      }
      if mo.quantity < PRODUCT_MINIMUM_INVENTORY_QUANTITY {
        let params = Some(HashMap::from([(
          "Min".into(),
          Value::Number(PRODUCT_MINIMUM_INVENTORY_QUANTITY.into()),
        )]));
        let err = AppErrorError { id: "form.field.min".into(), params };
        field_error(errors, step, form_id, &format!("minimum_orders.{}.quantity", mo.id), err);
      }

      match mo.price.parse::<f64>() {
        Ok(price) => {
          if price <= 0.0 {
            let f_name = &format!("minimum_orders.{}.price", mo.id);
            field_error(errors, step, form_id, f_name, ERR_GT_0.clone());
          }
        }
        Err(_) => {
          let f_name = &format!("minimum_orders.{}.price", mo.id);
          field_error(errors, step, form_id, f_name, ERR_INVALID_NUM.clone());
        }
      }
    }
  }
}

fn validate_safety_form(
  form: ProductCreateRequestSafety,
  errors: &mut HashMap<String, AppErrorError>,
  sub: &Subcategory,
) {
  let step = &ProductCreateStepsNames::Safety;
  if form.form.len() == 0 {
    errors.insert(
      missing_form(step),
      AppErrorError { id: "products.safety_and_compliance.form.missing".into(), params: None },
    );
    return;
  }

  let default_validation = &ValidationField::default();
  if !form.attestation {
    field_error(errors, step, None, "attestation", ERR_MUST_CHECK.clone());
  }

  for (field_name, field_value) in form.form.iter() {
    let field = sub.safety.get(field_name);
    if field.is_none() {
      unknown_field(errors, &step, None, &field_name);
      break;
    }

    let field = field.unwrap();
    validate_attribute(
      field_value,
      field_name,
      field.r#type.as_ref(),
      errors,
      step,
      None,
      field.validation.as_ref().unwrap_or(default_validation),
      field.required,
      field.string_array.as_ref(),
    );
  }
}

// TODO: handle is_multiple for the select type
fn validate_attribute(
  field_value: &Any,
  field_name: &str,
  typ: &str,
  errors: &mut HashMap<String, AppErrorError>,
  step: &ProductCreateStepsNames,
  form_id: Option<&str>,
  validation: &ValidationField,
  required: bool,
  string_array: &Vec<String>,
) {
  let value = grpc_deserialize_any(field_value);
  match SubcategoryAttributeType::from_str(&typ) {
    SubcategoryAttributeType::Input => {
      match value {
        AnyValue::String(val) => {
          // if rule is none, than this field is incorrect or manipulated
          // don't put it above!, because E.g. select has no validation(so far)
          if validation.rule.is_none() {
            invalid_field_data(errors, step, form_id, field_name);
            return;
          }
          match validation.rule.as_ref().unwrap() {
            Rule::Numeric(_) => {
              invalid_field_data(errors, step, form_id, field_name);
              return;
            }
            Rule::Str(s) => {
              validate_string(&s, val, errors, form_id, field_name, step);
            }
            Rule::Regex(r) => {
              validate_regex(&r, val, errors, form_id, field_name, step);
            }
          }
        }
        AnyValue::Double(num) => match validation.rule.as_ref().unwrap() {
          Rule::Numeric(val) => {
            validate_numeric(&val, num, errors, form_id, field_name, step);
          }
          _ => {
            invalid_field_data(errors, step, form_id, field_name);
          }
        },
        AnyValue::Int32(int32) => match validation.rule.as_ref().unwrap() {
          Rule::Numeric(n) => {
            validate_numeric(&n, int32 as f64, errors, form_id, field_name, step);
          }
          _ => {
            invalid_field_data(errors, step, form_id, field_name);
          }
        },
        AnyValue::Int64(int64) => match validation.rule.as_ref().unwrap() {
          Rule::Numeric(n) => {
            validate_numeric(&n, int64 as f64, errors, form_id, field_name, step);
          }
          _ => {
            invalid_field_data(errors, step, form_id, field_name);
          }
        },
        // Float, Bool, Bytes, Unknown can't happen for input type
        _ => {
          invalid_field_data(errors, step, form_id, field_name);
        }
      }
    }
    SubcategoryAttributeType::Select => match value {
      AnyValue::String(v) => {
        if required && v.is_empty() {
          field_error(errors, step, form_id, &field_name, ERR_REQUIRED.clone());
        }
        if !string_array.contains(&v) {
          field_error(errors, step, form_id, &field_name, ERR_INVALID_INP.clone());
        }
      }
      _ => {
        invalid_field_data(errors, step, form_id, field_name);
      }
    },
    SubcategoryAttributeType::Boolean => match value {
      AnyValue::Bool(val) => {
        if required && !val {
          field_error(errors, step, form_id, &field_name, ERR_MUST_CHECK.clone());
        }
      }
      _ => {
        invalid_field_data(errors, step, form_id, field_name);
      }
    },
    SubcategoryAttributeType::Unknown => {
      invalid_field_data(errors, step, form_id, field_name);
    }
  }
}

fn validate_string(
  validation: &ValidationFieldString,
  value: String,
  errors: &mut HashMap<String, AppErrorError>,
  form_id: Option<&str>,
  field_name: &str,
  step_name: &ProductCreateStepsNames,
) {
  for rule in validation.rules.iter() {
    if rule.r#type == (StringRuleType::Min as i32) {
      if value.len() < (rule.value as usize) {
        let err = AppErrorError {
          id: "form.field.min_length".to_string(),
          params: Some(HashMap::from([("Min".to_string(), json!(rule.value))])),
        };
        field_error(errors, &step_name, form_id, field_name, err);
      }
    } else if rule.r#type == (StringRuleType::Max as i32) {
      if value.len() > (rule.value as usize) {
        let err = AppErrorError {
          id: "form.field.max_length".to_string(),
          params: Some(HashMap::from([("Max".to_string(), json!(rule.value))])),
        };
        field_error(errors, &step_name, form_id, field_name, err);
      }
    }
  }
}

fn validate_numeric(
  validation: &ValidationFieldNumeric,
  value: f64,
  errors: &mut HashMap<String, AppErrorError>,
  form_id: Option<&str>,
  field_name: &str,
  step: &ProductCreateStepsNames,
) {
  for rule in validation.rules.iter() {
    if rule.r#type == (NumericRuleType::Min as i32) {
      if value < rule.value {
        let err = AppErrorError {
          id: "form.field.min".to_string(),
          params: Some(HashMap::from([("Min".to_string(), json!(rule.value))])),
        };
        field_error(errors, step, form_id, field_name, err);
      }
    } else if rule.r#type == (NumericRuleType::Max as i32) {
      if value > rule.value {
        let err = AppErrorError {
          id: "form.field.max".to_string(),
          params: Some(HashMap::from([("Max".to_string(), json!(rule.value))])),
        };
        field_error(errors, step, form_id, field_name, err);
      }
    } else if rule.r#type == (NumericRuleType::Gt as i32) {
      if value <= rule.value {
        let err = AppErrorError {
          id: "form.field.greater_than".to_string(),
          params: Some(HashMap::from([("Max".to_string(), json!(rule.value))])),
        };
        field_error(errors, step, form_id, field_name, err);
      }
    } else if rule.r#type == (NumericRuleType::Lt as i32) {
      if value >= rule.value {
        let err = AppErrorError {
          id: "form.field.less_than".to_string(),
          params: Some(HashMap::from([("Min".to_string(), json!(rule.value))])),
        };
        field_error(errors, step, form_id, field_name, err);
      }
    }
  }
}

// TODO: implement the validation logic
fn validate_regex(
  _validation: &ValidationFieldRegex,
  _value: String,
  _errors: &mut HashMap<String, AppErrorError>,
  _form_id: Option<&str>,
  field_name: &str,
  _step_name: &ProductCreateStepsNames,
) {
  panic!(
    "the attribute: {} is of type Regex, and validation logic is not implemented in: {}",
    field_name, "products.models.details_form_validation"
  );
}

fn invalid_field_data(
  errors: &mut HashMap<String, AppErrorError>,
  form_name: &ProductCreateStepsNames,
  form_id: Option<&str>,
  field_name: &str,
) {
  let key = match form_id {
    Some(fid) => format!("{}.{}.{}", form_name.as_str(), fid, field_name),
    None => format!("{}.{}", form_name.as_str(), field_name),
  };
  let params =
    Some(HashMap::from([("FieldName".to_string(), Value::String(field_name.to_string()))]));
  errors.insert(key, AppErrorError { id: "form.field.invalid_data".to_string(), params });
}

fn field_error(
  errors: &mut HashMap<String, AppErrorError>,
  form_name: &ProductCreateStepsNames,
  form_id: Option<&str>,
  field_name: &str,
  err: AppErrorError,
) {
  let key = match form_id {
    Some(fid) => format!("{}.{}.{}", form_name.as_str(), fid, field_name),
    None => format!("{}.{}", form_name.as_str(), field_name),
  };
  errors.insert(key, err);
}

fn unknown_field(
  errors: &mut HashMap<String, AppErrorError>,
  form_name: &ProductCreateStepsNames,
  form_id: Option<&str>,
  field_name: &str,
) {
  let params = HashMap::from([("FieldName".to_string(), Value::String(field_name.to_string()))]);
  let err = AppErrorError { id: "form.field.unknown".into(), params: Some(params) };
  let key = match form_id {
    Some(fid) => format!("{}.{}.{}", form_name.as_str(), fid, field_name),
    None => format!("{}.{}", form_name.as_str(), field_name),
  };
  errors.insert(key, err);
}

fn missing_form_id(form_name: &ProductCreateStepsNames) -> String {
  format!("{}.form_id.missing", form_name.as_str())
}

fn missing_form(form_name: &ProductCreateStepsNames) -> String {
  format!("{}.form.missing", form_name.as_str())
}

fn error_builder(ctx: Arc<Context>, errors: HashMap<String, AppErrorError>) -> AppError {
  // for err in errors.iter() {
  //   println!("field {}, has error {:#?}", err.0, err.1);
  // }
  AppError::new(
    ctx,
    "products.models.products_create_is_valid",
    "form.fields.invalid",
    None,
    "".to_string(),
    Code::InvalidArgument.into(),
    Some(AppErrorErrors { errors_internal: Some(errors), ..Default::default() }),
  )
}

pub fn products_create_auditable_v1(product: &ProductCreateRequest) -> Value {
  let identity = product.identity.clone().unwrap_or_default();
  let description = product.description.clone().unwrap_or_default();
  let details = product.details.clone().unwrap_or_default();
  let media = product.media.clone().unwrap_or_default();
  let offer = product.offer.clone().unwrap_or_default();
  let safety = product.safety.clone().unwrap_or_default();

  let mut media_var =
    ProductCreateRequestMediaWithVariants { images: HashMap::new(), videos: HashMap::new() };
  let mut media_no_var =
    ProductCreateRequestMediaWithoutVariants { images: vec![], videos: vec![] };
  let mut med = ProductCreateRequestMedia { total_size: media.total_size, media: None };

  if let Some(media) = media.media {
    match media {
      MediaWithVariants(m) => {
        media_var.images = product_create_auditable_attachments_with_variants_v1(m.images);
        media_var.videos = product_create_auditable_attachments_with_variants_v1(m.videos);
        med.media = Some(MediaWithVariants(media_var));
      }
      MediaNoVariants(m) => {
        media_no_var.images = product_create_auditable_attachments_no_variants_v1(m.images);
        media_no_var.videos = product_create_auditable_attachments_no_variants_v1(m.videos);
        med.media = Some(MediaNoVariants(media_no_var));
      }
    }
  }

  json!({
    "identity": identity,
    "description": description,
    "details": details,
    "media": med,
    "offer": offer,
    "safety": safety,
  })
}

fn product_create_auditable_attachments_with_variants_v1(
  attachments: HashMap<String, Attachments>,
) -> HashMap<String, Attachments> {
  let mut result: HashMap<String, Attachments> = HashMap::new();
  for form in attachments.iter() {
    let mut items: Vec<Attachment> = vec![];
    for attachment in form.1.attachments.iter() {
      items.push(Attachment { base64: "".to_string(), data: vec![], ..attachment.clone() });
    }
    result.insert(form.0.to_string(), Attachments { attachments: items });
  }
  result
}

fn product_create_auditable_attachments_no_variants_v1(
  attachments: Vec<Attachment>,
) -> Vec<Attachment> {
  let mut result: Vec<Attachment> = vec![];
  for att in attachments.iter() {
    result.push(Attachment { base64: "".to_string(), data: vec![], ..att.clone() });
  }
  result
}

#[derive(Debug)]
pub struct ProductCreatePreSaveResult {
  pub product: Product,
  pub variants_ids: HashMap<String, String>,
  pub main_variant_key: String,
}

pub fn products_create_pre_save(
  ctx: Arc<Context>,
  pro: &ProductCreateRequest,
) -> Result<ProductCreatePreSaveResult, AppError> {
  let which = "which must not occurr at all, because it got validated before!";

  let created_at = time_get_millis();
  let variant_id = created_at.to_string();
  let identity = pro.identity.clone().unwrap();
  let description = pro.description.clone().unwrap();
  let bullet_points: Vec<ProductBulletPoint> = description
    .bullet_points
    .iter()
    .map(|bp| ProductBulletPoint {
      id: Ulid::new().to_string(),
      created_at,
      text: bp.bullet_point.clone(),
      updated_at: None,
    })
    .collect();

  let mut variant_ids: HashMap<String, String> = HashMap::new();

  let details =
    products_create_pre_save_details(&pro.details, &mut variant_ids, &variant_id, which);
  let offer = products_create_pre_save_offer(&pro.offer, &variant_ids, &variant_id, created_at);
  let safety = products_create_pre_save_safety(&pro.safety);

  let off = pro.offer.as_ref().unwrap();
  let product = Product {
    id: ulid::Ulid::new().to_string(),
    user_id: ctx.session().user_id().to_string(),
    title: identity.title.clone(),
    category: identity.category,
    subcategory: identity.subcategory,
    has_variations: identity.has_variations,
    brand_name: if identity.no_brand { None } else { Some(identity.brand_name) },
    has_brand_name: identity.no_brand,
    product_id: if identity.no_product_id { None } else { Some(identity.product_id) },
    product_id_type: if identity.no_product_id { None } else { Some(identity.product_id_type) },
    has_product_id: identity.no_product_id,
    description: description.description,
    bullet_points,
    currency_code: off.currency.clone(),
    fulfillment_type: off.fulfillment_type.clone(),
    processing_time: off.processing_time,
    details: Some(details),
    media: None,
    offer: Some(offer),
    safety: Some(safety),
    tags: vec![],
    metadata: None,
    ar_enabled: false,
    slug: Slug::default().generate_slug(&identity.title),
    status: ProductStatus::Pending.as_string(),
    version: 1,
    schema_version: 1,
    created_at,
    published_at: None,
    updated_at: None,
  };

  Ok(ProductCreatePreSaveResult {
    product,
    variants_ids: variant_ids,
    main_variant_key: variant_id,
  })
}

fn products_create_pre_save_details(
  details: &Option<ProductCreateRequestDetails>,
  variant_ids: &mut HashMap<String, String>,
  variant_id: &str,
  which: &str,
) -> ProductDetails {
  let mut details_var: HashMap<String, ProductDetailsVariant> = HashMap::new();
  let mut shared: HashMap<String, Any> = HashMap::new();
  let path = "products.models.products_create_pre_save_details";

  match details {
    Some(details_outer) => {
      match &details_outer.details {
        Some(details) => match details {
          DetailsVariants(forms) => {
            for form in forms.variants.iter() {
              let var_id_new = Ulid::new().to_string();
              let var_data = get_variant_id_and_title(&form.form, path, which);
              variant_ids.insert(var_data.0, var_id_new.clone());
              details_var.insert(
                var_id_new,
                ProductDetailsVariant { variant_name: var_data.1, variant_data: form.form.clone() },
              );
            }
            shared.extend(details_outer.shared.clone());
          }
          DetailsNoVariants(form) => {
            // TODO: consider wither we need the variant name to be empty for a product
            // without variants, or we can ask the user to name this
            let var_id = Ulid::new().to_string();
            variant_ids.insert(variant_id.to_string(), var_id.clone());
            details_var.insert(
              var_id,
              ProductDetailsVariant {
                variant_name: "".to_string(),
                variant_data: form.form.clone(),
              },
            );
          }
        },
        None => {
          panic!("{}: the product's ProductCreateRequestDetails.details form is missing!", &path);
        }
      };
    }
    None => {
      panic!("{}: the product's details form is missing!", &path);
    }
  }

  ProductDetails { shared, details: details_var }
}

fn get_variant_id_and_title(
  form: &HashMap<String, Any>,
  path: &str,
  which: &str,
) -> (String, String) {
  let get_string_field = |field: &str| -> String {
    match grpc_deserialize_any(form.get(field).unwrap()) {
      AnyValue::String(s) => s,
      value => panic!(
        "{}: the product's variant {} is not a valid string or missing, {}: {}",
        path, field, which, value
      ),
    }
  };

  let var_id = get_string_field("id");
  let var_name = get_string_field("title");

  (var_id, var_name)
}

fn products_create_pre_save_offer(
  offer: &Option<ProductCreateRequestOffer>,
  variant_ids: &HashMap<String, String>,
  variant_id: &str,
  created_at: u64,
) -> ProductOffer {
  let mut result = ProductOffer { offer: HashMap::new() };
  let path = "products.models.products_create_pre_save_offer".to_string();

  let get_min_orders =
    |mo: &Vec<ProductCreateRequestOfferMinimumOrder>| -> Vec<ProductOfferMinimumOrder> {
      mo.iter()
        .map(|mo| ProductOfferMinimumOrder {
          id: Ulid::new().to_string(),
          created_at,
          updated_at: None,
          price: mo.price.clone(),
          quantity: mo.quantity,
        })
        .collect()
    };

  match offer {
    Some(offer_inner) => match &offer_inner.pricing {
      Some(pricing) => match pricing {
        OfferWithVariants(form) => {
          for var in form.variants.iter() {
            let var_id = variant_ids
              .get(&var.id)
              .expect(&format!("{}: the product variant's id is not found", path));

            let has_sale = var.has_sale_price.unwrap_or_default();
            result.offer.insert(
              var_id.to_string(),
              ProductOfferVariant {
                sku: var.sku.clone(),
                quantity: var.quantity,
                price: var.price.clone(),
                offering_condition: var.offering_condition.clone(),
                condition_note: var.condition_note.clone(),
                list_price: var.list_price.clone(),
                has_sale_price: has_sale,
                sale_price: if has_sale { var.sale_price.clone() } else { None },
                sale_price_start: if has_sale { var.sale_price_start.clone() } else { None },
                sale_price_end: if has_sale { var.sale_price_end.clone() } else { None },
                has_minimum_orders: var.has_minimum_orders,
                minimum_orders: get_min_orders(&var.minimum_orders),
              },
            );
          }
        }
        OfferNoVariants(var) => {
          let var_id = variant_ids
            .get(variant_id)
            .expect(&format!("{}: the product variant's id is not found", path));
          let has_sale = var.has_sale_price.unwrap_or_default();
          result.offer.insert(
            var_id.to_string(),
            ProductOfferVariant {
              sku: var.sku.clone(),
              quantity: var.quantity,
              price: var.price.clone(),
              offering_condition: var.offering_condition.clone(),
              condition_note: var.condition_note.clone(),
              list_price: var.list_price.clone(),
              has_sale_price: has_sale,
              sale_price: if has_sale { var.sale_price.clone() } else { None },
              sale_price_start: if has_sale { var.sale_price_start.clone() } else { None },
              sale_price_end: if has_sale { var.sale_price_end.clone() } else { None },
              has_minimum_orders: var.has_minimum_orders,
              minimum_orders: get_min_orders(&var.minimum_orders),
            },
          );
        }
      },
      None => {
        panic!("{}: the product's offer form is missing!", &path);
      }
    },
    None => todo!(),
  }

  return result;
}

fn products_create_pre_save_safety(safety: &Option<ProductCreateRequestSafety>) -> ProductSafety {
  let mut result = ProductSafety { safety: HashMap::new() };
  let path = "products.models.products_create_pre_save_safety".to_string();
  match &safety {
    Some(safety) => {
      result.safety.extend(safety.form.clone());
      result.safety.insert("attestation".to_string(), Any::from_bool(safety.attestation));
    }
    None => {
      panic!("{}: the product's offer form is missing!", &path);
    }
  }

  result
}
