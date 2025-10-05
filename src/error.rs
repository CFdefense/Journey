use axum::http::StatusCode;

pub type ApiResult<T> = std::result::Result<T, StatusCode>;

/// Added to fix tarpaulin bug. This can be removed once other things are added to this file
enum Error {}