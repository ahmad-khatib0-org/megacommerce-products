use regex::Regex;
use sqlx::error::Error as SqlxError;
use sqlx::postgres::PgDatabaseError;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DBErrorType {
  NoRows,
  UniqueViolation,
  ForeignKeyViolation,
  NotNullViolation,
  JsonMarshal,
  JsonUnmarshal,
  Connection,
  Privileges,
  Internal,
}

impl fmt::Display for DBErrorType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DBErrorType::NoRows => write!(f, "no_rows"),
      DBErrorType::UniqueViolation => write!(f, "unique_violation"),
      DBErrorType::ForeignKeyViolation => write!(f, "foreign_key_violation"),
      DBErrorType::NotNullViolation => write!(f, "not_null_violation"),
      DBErrorType::JsonMarshal => write!(f, "json_marshal"),
      DBErrorType::JsonUnmarshal => write!(f, "json_unmarshal"),
      DBErrorType::Connection => write!(f, "connection_exception"),
      DBErrorType::Privileges => write!(f, "insufficient_privilege"),
      DBErrorType::Internal => write!(f, "internal_error"),
    }
  }
}

#[derive(Debug)]
pub struct DBError {
  pub err_type: DBErrorType,
  pub err: Option<Box<dyn Error + Send + Sync>>,
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

    if let Some(ref err) = self.err {
      parts.push(format!("err: {}", err));
    }

    write!(f, "{}", parts.join(", "))
  }
}

impl Error for DBError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    self.err.as_ref().map(|e| &**e as &dyn Error)
  }
}

impl DBError {
  pub fn new(
    err_type: DBErrorType,
    err: Option<Box<dyn Error + Send + Sync>>,
    msg: impl Into<String>,
    path: impl Into<String>,
    details: impl Into<String>,
  ) -> Self {
    Self { err_type, err, msg: msg.into(), path: path.into(), details: details.into() }
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
        "23505" => DBErrorType::UniqueViolation,
        "23503" => DBErrorType::ForeignKeyViolation,
        "23502" => DBErrorType::NotNullViolation,
        "08000" | "08003" | "08006" => DBErrorType::Connection,
        "42501" => DBErrorType::Privileges,
        _ => DBErrorType::Internal,
      };

      DBError::new(err_type, Some(Box::new(SqlxError::Database(db_err))), msg, path, details)
    }

    SqlxError::RowNotFound => DBError::new(
      DBErrorType::NoRows,
      Some(Box::new(SqlxError::RowNotFound)),
      "the requested resource is not found",
      path,
      "",
    ),

    _ => DBError::new(DBErrorType::Internal, Some(Box::new(err)), "database error", path, ""),
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
