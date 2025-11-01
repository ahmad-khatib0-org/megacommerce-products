use std::{collections::HashMap, sync::Arc};

use megacommerce_proto::{
  product_create_request_details::Details::{
    WithVariants as DetailsVariants, WithoutVariants as DetailsNoVariants,
  },
  validation_field::Rule,
  Any, NumericRuleType, Product, ProductCreateRequest, ProductCreateRequestDescription,
  ProductCreateRequestDetails, ProductCreateRequestDetailsVariantForm,
  ProductCreateRequestIdentity, ProductCreateRequestMedia, ProductCreateRequestOffer,
  ProductCreateRequestSafety, ProductDataResponseSubcategory, StringRuleType, Subcategory,
  SubcategoryAttribute, ValidationField, ValidationFieldNumeric, ValidationFieldRegex,
  ValidationFieldString,
};
use megacommerce_shared::{
  models::{
    context::Context,
    errors::{AppError, AppErrorError, AppErrorErrors},
    products::SubcategoryAttributeType,
  },
  utils::grpc::{grpc_deserialize_any, AnyValue},
};
use serde_json::{json, Value};
use tonic::Code;

use crate::{
  models::products::{
    product_id_is_validate, ProductCreateStepsNames, PRODUCT_BRAND_NAME_MAX_LENGTH,
    PRODUCT_BRAND_NAME_MIN_LENGTH, PRODUCT_DESCRIPTION_BULLET_POINTS_MAX_LENGTH,
    PRODUCT_DESCRIPTION_BULLET_POINTS_MIN_LENGTH, PRODUCT_DESCRIPTION_BULLET_POINT_MAX_LENGTH,
    PRODUCT_DESCRIPTION_BULLET_POINT_MIN_LENGTH, PRODUCT_DESCRIPTION_MAX_LENGTH,
    PRODUCT_DESCRIPTION_MIN_LENGTH, PRODUCT_ID_TYPES, PRODUCT_TITLE_MAX_LENGTH,
    PRODUCT_TITLE_MIN_LENGTH,
  },
  utils::time::time_get_millis,
};

use super::products::ProductStatus;

pub fn products_create_is_valid(
  ctx: Arc<Context>,
  product: &ProductCreateRequest,
  subcategory_data: Option<ProductDataResponseSubcategory>,
) -> Result<(), AppError> {
  let identity = product.identity.clone().unwrap_or(ProductCreateRequestIdentity::default());
  let description =
    product.description.clone().unwrap_or(ProductCreateRequestDescription::default());
  let details = product.details.clone().unwrap_or(ProductCreateRequestDetails::default());
  let media = product.media.clone().unwrap_or(ProductCreateRequestMedia::default());
  let offer = product.offer.clone().unwrap_or(ProductCreateRequestOffer::default());
  let safety = product.safety.clone().unwrap_or(ProductCreateRequestSafety::default());
  let mut errors: HashMap<String, AppErrorError> = HashMap::new();

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
    && product_id_is_validate(&identity.product_id_type, &identity.product_id)
  {
    errors.insert(
      "identity.product_id".into(),
      AppErrorError { id: "products.external_product_id.invalid".into(), params: None },
    );
  }

  // fail early
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

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
  if errors.len() > 0 {
    return Err(error_builder(ctx, errors));
  }

  let sub_data = subcategory_data.unwrap();
  let sub = sub_data.data.unwrap();
  let trans = sub_data.translations.unwrap();

  if details.details.is_none() {
    errors.insert(
      "details.form.missing".into(),
      AppErrorError { id: "products.details.form.missing".into(), params: None },
    );
    return Err(error_builder(ctx, errors));
  }

  let details_form = details.details.unwrap();
  match details_form {
    DetailsVariants(form) => {
      for variant in form.variants.iter() {
        details_form_validation(variant, &mut errors, &sub);
      }
    }
    DetailsNoVariants(form) => {
      let n = form.form.len();
    }
  };

  // let mut valid_currency = false;
  // for c in CURRENCY_LIST {
  //   if c.code == offer.currency {
  //     valid_currency = true;
  //     break;
  //   }
  // }
  // if !valid_currency {
  //   return Err(error_builder(ctx, "currency_code", offer.currency, None));
  // }

  // if sku.chars().count() < PRODUCT_SKU_MIN_LENGTH || sku.chars().count() > PRODUCT_SKU_MAX_LENGTH {
  //   let p = HashMap::from([
  //     ("Min".to_string(), Value::Number(PRODUCT_SKU_MIN_LENGTH.into())),
  //     ("Max".to_string(), Value::Number(PRODUCT_SKU_MAX_LENGTH.into())),
  //   ]);
  //   return Err(error_builder(ctx, "sku", title, Some(p)));
  // }

  // let price_err = || {
  //   error_builder(
  //     ctx.clone(),
  //     "price.invalid",
  //     price,
  //     Some(HashMap::from([("Price".to_string(), Value::String(price.clone()))])),
  //   )
  // };
  // let parsed_price = price.parse::<f64>().map_err(|_| price_err())?;
  // if parsed_price <= 0.0 {
  //   return Err(price_err());
  // }

  Ok(())
}

// TODO: check for the fields that are required in &Subcategory, but user didn't send them
fn details_form_validation(
  variant: &ProductCreateRequestDetailsVariantForm,
  errors: &mut HashMap<String, AppErrorError>,
  sub: &Subcategory,
) {
  let form_id = grpc_deserialize_any(variant.form.get("id").unwrap_or(&Any::default()));
  let id = match form_id {
    AnyValue::String(form_id) => form_id,
    _ => "".to_string(),
  };
  if id.is_empty() {
    errors.insert(
      missing_field(&ProductCreateStepsNames::Details),
      AppErrorError { id: "form.field.id.missing_or_invalid".to_string(), params: None },
    );
    return;
  }

  let step = ProductCreateStepsNames::Details;
  for (field_name, field_value) in variant.form.iter() {
    let found_field = sub.attributes.get(field_name);
    if found_field.is_none() && field_name.as_str() != "id" && field_name.as_str() != "title" {
      let key = unknown_field(&ProductCreateStepsNames::Details, &id, field_name);
      let val = Value::String(field_name.to_string());
      let params = Some(HashMap::from([("FieldName".to_string(), val)]));
      errors.insert(key, AppErrorError { id: "form.field.unknown".into(), params });
      break;
    }

    validate_attribute(errors, &step, Some(&id), found_field, field_name, field_value);
  }
}

fn validate_attribute(
  errors: &mut HashMap<String, AppErrorError>,
  step: &ProductCreateStepsNames,
  form_id: Option<&str>,
  found_field: Option<&SubcategoryAttribute>,
  field_name: &str,
  field_value: &Any,
) {
  let field = found_field.unwrap();
  let typ = field.r#type.clone();
  let required = field.required;
  let string_array = field.string_array.clone();
  let validation = field.validation.clone().unwrap_or(ValidationField::default());
  // let include = field.include_in_variants;
  // let is_multiple = field.is_multiple;

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
          match validation.rule.unwrap() {
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
        AnyValue::Double(num) => match validation.rule.unwrap() {
          Rule::Numeric(val) => {
            validate_numeric(&val, num, errors, form_id, field_name, step);
          }
          _ => {
            invalid_field_data(errors, step, form_id, field_name);
            return;
          }
        },
        AnyValue::Int64(int) => match validation.rule.unwrap() {
          Rule::Numeric(n) => {
            validate_numeric(&n, int as f64, errors, form_id, field_name, step);
          }
          _ => {
            invalid_field_data(errors, step, form_id, field_name);
            return;
          }
        },
        // Float, Int32, Bool, Bytes, Unknown can't happen for input type
        _ => {
          invalid_field_data(errors, step, form_id, field_name);
          return;
        }
      }
    }
    SubcategoryAttributeType::Select => match value {
      AnyValue::String(v) => {
        if required && v.is_empty() {
          let err = AppErrorError { id: "required".to_string(), params: None };
          field_error(errors, step, form_id, &field_name, err);
        }
        if !string_array.contains(&v) {
          let err = AppErrorError { id: "form.field.invalid_input".to_string(), params: None };
          field_error(errors, step, form_id, &field_name, err);
        }
      }
      _ => {
        invalid_field_data(errors, step, form_id, field_name);
        return;
      }
    },
    SubcategoryAttributeType::Boolean => match value {
      AnyValue::Bool(val) => {
        if required && !val {
          let err = AppErrorError { id: "form.field.checkbox.checked.error".into(), params: None };
          field_error(errors, step, form_id, &field_name, err);
        }
      }
      _ => {
        invalid_field_data(errors, step, form_id, field_name);
        return;
      }
    },
    SubcategoryAttributeType::Unknown => {
      invalid_field_data(errors, step, form_id, field_name);
      return;
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
        field_error(errors, &step_name, form_id, &field_name, err);
      }
    } else if rule.r#type == (StringRuleType::Max as i32) {
      if value.len() > (rule.value as usize) {
        let err = AppErrorError {
          id: "form.field.max_length".to_string(),
          params: Some(HashMap::from([("Max".to_string(), json!(rule.value))])),
        };
        field_error(errors, &step_name, form_id, &field_name, err);
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
  step_name: &ProductCreateStepsNames,
) {
  for rule in validation.rules.iter() {
    if rule.r#type == (NumericRuleType::Min as i32) {
      if value < rule.value {
        let err = AppErrorError {
          id: "form.field.min".to_string(),
          params: Some(HashMap::from([("Min".to_string(), json!(rule.value))])),
        };
        field_error(errors, step_name, form_id, &field_name, err);
      }
    } else if rule.r#type == (NumericRuleType::Max as i32) {
      if value > rule.value {
        let err = AppErrorError {
          id: "form.field.max".to_string(),
          params: Some(HashMap::from([("Max".to_string(), json!(rule.value))])),
        };
        field_error(errors, step_name, form_id, &field_name, err);
      }
    } else if rule.r#type == (NumericRuleType::Gt as i32) {
      if value <= rule.value {
        let err = AppErrorError {
          id: "form.field.greater_than".to_string(),
          params: Some(HashMap::from([("Max".to_string(), json!(rule.value))])),
        };
        field_error(errors, step_name, form_id, &field_name, err);
      }
    } else if rule.r#type == (NumericRuleType::Lt as i32) {
      if value >= rule.value {
        let err = AppErrorError {
          id: "form.field.less_than".to_string(),
          params: Some(HashMap::from([("Min".to_string(), json!(rule.value))])),
        };
        field_error(errors, step_name, form_id, &field_name, err);
      }
    }
  }
}

fn validate_regex(
  _validation: &ValidationFieldRegex,
  _value: String,
  _errors: &mut HashMap<String, AppErrorError>,
  _form_id: Option<&str>,
  field_name: &str,
  _step_name: &ProductCreateStepsNames,
) {
  // TODO: implement the validation logic
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
    Some(fid) => format!("{}.form.{}.{}", form_name.as_str(), fid, field_name),
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

fn unknown_field(form_name: &ProductCreateStepsNames, form_id: &str, field_name: &str) -> String {
  format!("{}.form.{}.{}", form_name.as_str(), form_id, field_name)
}

fn missing_field(form_name: &ProductCreateStepsNames) -> String {
  format!("{}.form_id.missing", form_name.as_str())
}

pub fn products_create_pre_save(
  ctx: Arc<Context>,
  pro: &ProductCreateRequest,
) -> Result<Product, AppError> {
  let id = ulid::Ulid::new().to_string();
  Ok(Product {
    id,
    user_id: ctx.session().user_id().to_string(),
    version: 1,
    status: ProductStatus::Pending.as_string(),
    metadata: None,
    created_at: time_get_millis(),
    published_at: None,
    updated_at: None,
    ..Default::default()
  })
}

pub fn products_create_auditable(p: &ProductCreateRequest) -> Value {
  json!({})
}

fn error_builder(ctx: Arc<Context>, errors: HashMap<String, AppErrorError>) -> AppError {
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
