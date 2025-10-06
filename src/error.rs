use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;
use tracing::error;

pub type ApiResult<T> = std::result::Result<T, StatusCode>;

// Rich error type for internal logging and status mapping
#[derive(Debug)]
pub enum Error {
    Validation(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Cookie(String),
    Auth(String),
    Db(sqlx::Error),
    PasswordHash(argon2::password_hash::Error),
    Json(serde_json::Error),
    Env(std::env::VarError),
    Internal(String),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) | Error::Auth(_) | Error::Cookie(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Json(_) => StatusCode::BAD_REQUEST,
            Error::Db(_) | Error::PasswordHash(_) | Error::Env(_) | Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn log(&self) {
        match self {
            Error::Db(e) => error!(target: "api_error", kind = "db", error = ?e, "Database error"),
            Error::PasswordHash(e) => error!(target: "api_error", kind = "hash", error = ?e, "Password hashing error"),
            Error::Json(e) => error!(target: "api_error", kind = "json", error = ?e, "JSON error"),
            Error::Env(e) => error!(target: "api_error", kind = "env", error = ?e, "Env var error"),
            Error::Validation(m) => error!(target: "api_error", kind = "validation", message = %m),
            Error::BadRequest(m) => error!(target: "api_error", kind = "bad_request", message = %m),
            Error::Unauthorized(m) => error!(target: "api_error", kind = "unauthorized", message = %m),
            Error::Forbidden(m) => error!(target: "api_error", kind = "forbidden", message = %m),
            Error::NotFound(m) => error!(target: "api_error", kind = "not_found", message = %m),
            Error::Cookie(m) => error!(target: "api_error", kind = "cookie", message = %m),
            Error::Auth(m) => error!(target: "api_error", kind = "auth", message = %m),
            Error::Internal(m) => error!(target: "api_error", kind = "internal", message = %m),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(m) => write!(f, "validation error: {m}"),
            Error::BadRequest(m) => write!(f, "bad request: {m}"),
            Error::Unauthorized(m) => write!(f, "unauthorized: {m}"),
            Error::Forbidden(m) => write!(f, "forbidden: {m}"),
            Error::NotFound(m) => write!(f, "not found: {m}"),
            Error::Cookie(m) => write!(f, "cookie error: {m}"),
            Error::Auth(m) => write!(f, "auth error: {m}"),
            Error::Db(e) => write!(f, "db error: {e}"),
            Error::PasswordHash(e) => write!(f, "password hash error: {e}"),
            Error::Json(e) => write!(f, "json error: {e}"),
            Error::Env(e) => write!(f, "env error: {e}"),
            Error::Internal(m) => write!(f, "internal error: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<sqlx::Error> for Error { fn from(e: sqlx::Error) -> Self { Error::Db(e) } }
impl From<argon2::password_hash::Error> for Error { fn from(e: argon2::password_hash::Error) -> Self { Error::PasswordHash(e) } }
impl From<serde_json::Error> for Error { fn from(e: serde_json::Error) -> Self { Error::Json(e) } }
impl From<std::env::VarError> for Error { fn from(e: std::env::VarError) -> Self { Error::Env(e) } }

impl From<Error> for StatusCode {
    fn from(e: Error) -> Self { e.status_code() }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.log();
        self.status_code().into_response()
    }
}