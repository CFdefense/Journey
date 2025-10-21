pub mod account;
pub mod itinerary;
pub mod chat;

#[cfg(any(test, not(debug_assertions)))]
pub type AxumRouter = axum::Router;
#[cfg(all(not(test), debug_assertions))]
pub type AxumRouter = utoipa_axum::router::OpenApiRouter;