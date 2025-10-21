pub mod account;
pub mod itinerary;
pub mod chat;

/// A regular [axum::Router] in test and release builds, or [utoipa_axum::router::OpenApiRouter] in non-test or dev builds
#[cfg(any(test, not(debug_assertions)))]
pub type AxumRouter = axum::Router;
/// A regular [axum::Router] in test and release builds, or [utoipa_axum::router::OpenApiRouter] in non-test or dev builds
#[cfg(all(not(test), debug_assertions))]
pub type AxumRouter = utoipa_axum::router::OpenApiRouter;