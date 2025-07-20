use regex::Regex;
use sqlx::error::Error as SqlxError;
use sqlx::postgres::PgDatabaseError;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tonic::Code;

use crate::models::context::Context;
use crate::models::errors::{AppError, ErrorType, InternalError, MSG_ID_ERR_INTERNAL};

#[derive(Debug)]
pub struct DBError {
  pub err_type: ErrorType,
  pub err: Box<dyn Error + Send + Sync>,
  pub msg: String,
  pub path: String,
  pub details: String,
}

impl fmt::Display for DBError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut parts = Vec::new();

    if !self.path.is_empty() {
      parts.push(format!("path: {}", self.path));
    }

    parts.push(format!("err_type: {}", self.err_type));

    if !self.msg.is_empty() {
      parts.push(format!("msg: {}", self.msg));
    }

    if !self.details.is_empty() {
      parts.push(format!("details: {}", self.details));
    }

    parts.push(format!("err: {}", self.err));

    write!(f, "{}", parts.join(", "))
  }
}

impl From<InternalError> for DBError {
  fn from(e: InternalError) -> Self {
    DBError { err_type: e.err_type, err: e.err, msg: e.msg, path: e.path, details: "".into() }
  }
}

impl Error for DBError {}

impl DBError {
  pub fn new(
    err_type: ErrorType,
    err: Box<dyn Error + Send + Sync>,
    msg: impl Into<String>,
    path: impl Into<String>,
    details: impl Into<String>,
  ) -> Self {
    Self { err_type, err, msg: msg.into(), path: path.into(), details: details.into() }
  }

  pub fn to_app_error_internal(self, ctx: Arc<Context>, path: String) -> AppError {
    AppError::new(
      ctx,
      path,
      MSG_ID_ERR_INTERNAL,
      None,
      self.details,
      Some(Code::Internal.into()),
      Some(self.err),
    )
  }
}

pub fn handle_db_error(err: SqlxError, path: &str) -> DBError {
  match err {
    SqlxError::Database(db_err) => {
      let pg_err = db_err.downcast_ref::<PgDatabaseError>();

      // Extract details before moving db_err into the Box
      let details = pg_err.detail().unwrap_or("").to_string();
      let msg = match pg_err.code() {
        // Constraint violations
        "23505" => {
          // unique_violation
          parse_duplicate_field_db_error(pg_err)
        }
        "23503" => {
          // foreign_key_violation
          "referenced record is not found".to_string()
        }
        "23502" => {
          // not_null_violation
          format!("{} cannot be null", parse_db_field_name(pg_err))
        }
        // Connection/availability errors
        "08000" | "08003" | "08006" => "database connection exception".to_string(),
        // Permission errors
        "42501" => "insufficient permissions to perform an action".to_string(),
        _ => "database error".to_string(),
      };

      let err_type = match pg_err.code() {
        "23505" => ErrorType::UniqueViolation,
        "23503" => ErrorType::ForeignKeyViolation,
        "23502" => ErrorType::NotNullViolation,
        "08000" | "08003" | "08006" => ErrorType::Connection,
        "42501" => ErrorType::Privileges,
        _ => ErrorType::Internal,
      };

      DBError::new(err_type, Box::new(SqlxError::Database(db_err)), msg, path, details)
    }

    SqlxError::RowNotFound => DBError::new(
      ErrorType::NoRows,
      Box::new(SqlxError::RowNotFound),
      "the requested resource is not found",
      path,
      "",
    ),

    _ => DBError::new(ErrorType::Internal, Box::new(err), "database error", path, ""),
  }
}

// Extract the duplicate field from error detail
// Example: "Key (email)=(test@example.com) already exists.
fn parse_duplicate_field_db_error(err: &PgDatabaseError) -> String {
  if let Some(detail) = err.detail() {
    if let Some(parts) = detail.split(")=(").next() {
      let field = parts.trim_start_matches("Key (");
      return format!("{} already exists", field);
    }
  }
  err.detail().unwrap_or("").to_string()
}

// Extract field name from error message
// Example: "null value in column \"email\" violates not-null constraint
fn parse_db_field_name(err: &PgDatabaseError) -> String {
  let re = Regex::new(r#"column "(.+?)""#).unwrap();
  if let Some(captures) = re.captures(err.message()) {
    if let Some(match_) = captures.get(1) {
      return match_.as_str().to_string();
    }
  }
  "field".to_string()
}
