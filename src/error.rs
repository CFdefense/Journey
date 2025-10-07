use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;
use tracing::error;

// Unified API result type
pub type ApiResult<T> = std::result::Result<T, AppError>;

// Errors safe to expose to clients (public)
#[derive(Debug)]
#[cfg(not(tarpaulin_include))]
pub enum PublicError {
    Validation(String),
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict(String),
}

// Internal errors that should not leak details (private)
#[derive(Debug)]
#[cfg(not(tarpaulin_include))]
pub enum PrivateError {
    Db(sqlx::Error),
    PasswordHash(argon2::password_hash::Error),
    Json(serde_json::Error),
    Env(std::env::VarError),
    Internal(String),
}

// Wrapper type used by handlers
#[derive(Debug)]
#[cfg(not(tarpaulin_include))]
pub enum AppError {
    Public(PublicError),
    Private(PrivateError),
}

impl AppError {
    #[cfg(not(tarpaulin_include))]
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Public(e) => match e {
                PublicError::Validation(_) => StatusCode::BAD_REQUEST,
                PublicError::BadRequest(_) => StatusCode::BAD_REQUEST,
                PublicError::Unauthorized => StatusCode::UNAUTHORIZED,
                PublicError::Forbidden => StatusCode::FORBIDDEN,
                PublicError::NotFound => StatusCode::NOT_FOUND,
                PublicError::Conflict(_) => StatusCode::CONFLICT,
            },
            AppError::Private(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn log(&self) {
        match self {
            AppError::Public(e) => match e {
                PublicError::Validation(m) => error!(target: "api_error", kind = "validation", message = %m),
                PublicError::BadRequest(m) => error!(target: "api_error", kind = "bad_request", message = %m),
                PublicError::Unauthorized => error!(target: "api_error", kind = "unauthorized"),
                PublicError::Forbidden => error!(target: "api_error", kind = "forbidden"),
                PublicError::NotFound => error!(target: "api_error", kind = "not_found"),
                PublicError::Conflict(m) => error!(target: "api_error", kind = "conflict", message = %m),
            },
            AppError::Private(e) => match e {
                PrivateError::Db(err) => error!(target: "api_error", kind = "db", error = ?err, "Database error"),
                PrivateError::PasswordHash(err) => error!(target: "api_error", kind = "hash", error = ?err, "Password hashing error"),
                PrivateError::Json(err) => error!(target: "api_error", kind = "json", error = ?err, "JSON error"),
                PrivateError::Env(err) => error!(target: "api_error", kind = "env", error = ?err, "Env var error"),
                PrivateError::Internal(m) => error!(target: "api_error", kind = "internal", message = %m),
            },
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Public(e) => match e {
                PublicError::Validation(m) => write!(f, "validation error: {m}"),
                PublicError::BadRequest(m) => write!(f, "bad request: {m}"),
                PublicError::Unauthorized => write!(f, "unauthorized"),
                PublicError::Forbidden => write!(f, "forbidden"),
                PublicError::NotFound => write!(f, "not found"),
                PublicError::Conflict(m) => write!(f, "conflict: {m}"),
            },
            AppError::Private(e) => match e {
                PrivateError::Db(err) => write!(f, "db error: {err}"),
                PrivateError::PasswordHash(err) => write!(f, "password hash error: {err}"),
                PrivateError::Json(err) => write!(f, "json error: {err}"),
                PrivateError::Env(err) => write!(f, "env error: {err}"),
                PrivateError::Internal(m) => write!(f, "internal error: {m}"),
            },
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl std::error::Error for AppError {}

impl From<PublicError> for AppError { fn from(e: PublicError) -> Self { AppError::Public(e) } }
impl From<PrivateError> for AppError { fn from(e: PrivateError) -> Self { AppError::Private(e) } }
impl From<sqlx::Error> for AppError { fn from(e: sqlx::Error) -> Self { AppError::Private(PrivateError::Db(e)) } }
impl From<argon2::password_hash::Error> for AppError { fn from(e: argon2::password_hash::Error) -> Self { AppError::Private(PrivateError::PasswordHash(e)) } }
impl From<serde_json::Error> for AppError { fn from(e: serde_json::Error) -> Self { AppError::Private(PrivateError::Json(e)) } }
impl From<std::env::VarError> for AppError { fn from(e: std::env::VarError) -> Self { AppError::Private(PrivateError::Env(e)) } }

#[cfg(not(tarpaulin_include))]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log full details internally, return only status code outward
        self.log();
        self.status_code().into_response()
    }
}