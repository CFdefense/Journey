use tower_cookies::{Cookie, Key};

pub mod account;
pub mod itinerary;
pub mod chat;

#[cfg(test)]
pub type AxumRouter = axum::Router;
#[cfg(not(test))]
pub type AxumRouter = utoipa_axum::router::OpenApiRouter;

// #[cfg(test)]
// pub type CookieStore = tower_cookies::cookie::CookieJar;
// #[cfg(not(test))]
// pub type CookieStore = tower_cookies::Cookies;

// #[cfg(test)]
// pub fn private_add(cookies: &mut CookieStore, key: &Key, cookie: Cookie<'static>) {
// 	cookies.add(cookie)
// }
// #[cfg(not(test))]
// pub fn private_add(cookies: &mut CookieStore, key: &Key, cookie: Cookie<'static>) {
// 	cookies.private(key).add(cookie)
// }