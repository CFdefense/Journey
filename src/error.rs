use axum::http::StatusCode;

pub type ApiResult<T> = std::result::Result<T, StatusCode>;

