use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;
use tracing::error;

// Unified API result type
pub type ApiResult<T> = std::result::Result<T, AppError>;

// Single unified error for the API
#[derive(Debug)]
#[cfg(not(tarpaulin_include))]
pub enum AppError {
    Validation(String),
    BadRequest(String),
    Unauthorized,
    NotFound,
    Conflict(String),
    Internal(String),
}

impl AppError {
    #[cfg(not(tarpaulin_include))]
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn log(&self) {
        match self {
            AppError::Validation(m) => error!(target: "api_error", prefix = "ERROR ->>", kind = "validation", message = %m),
            AppError::BadRequest(m) => error!(target: "api_error", prefix = "ERROR ->>", kind = "bad_request", message = %m),
            AppError::Unauthorized => error!(target: "api_error", prefix = "ERROR ->>", kind = "unauthorized"),
            AppError::NotFound => error!(target: "api_error", prefix = "ERROR ->>", kind = "not_found"),
            AppError::Conflict(m) => error!(target: "api_error", prefix = "ERROR ->>", kind = "conflict", message = %m),
            AppError::Internal(m) => error!(target: "api_error", prefix = "ERROR ->>", kind = "internal", message = %m),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Validation(m) => write!(f, "validation error: {m}"),
            AppError::BadRequest(m) => write!(f, "bad request: {m}"),
            AppError::Unauthorized => write!(f, "unauthorized"),
            AppError::NotFound => write!(f, "not found"),
            AppError::Conflict(m) => write!(f, "conflict: {m}"),
            AppError::Internal(m) => write!(f, "internal error: {m}"),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl std::error::Error for AppError {}

// Convert common error types into unified Internal errors
impl From<sqlx::Error> for AppError { fn from(e: sqlx::Error) -> Self { AppError::Internal(format!("db error: {e:?}")) } }
impl From<argon2::password_hash::Error> for AppError { fn from(e: argon2::password_hash::Error) -> Self { AppError::Internal(format!("password hash error: {e:?}")) } }
impl From<serde_json::Error> for AppError { fn from(e: serde_json::Error) -> Self { AppError::Internal(format!("json error: {e:?}")) } }
impl From<std::env::VarError> for AppError { fn from(e: std::env::VarError) -> Self { AppError::Internal(format!("env error: {e:?}")) } }

#[cfg(not(tarpaulin_include))]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Always log; return only status code
        self.log();
        self.status_code().into_response()
    }
}