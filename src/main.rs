#![allow(unexpected_cfgs)]

mod controllers;
mod db;
mod error;
mod global;
mod http_models;
mod log;
mod middleware;
mod sql_models;

#[cfg(not(tarpaulin_include))]
mod swagger;

#[cfg(test)]
mod tests;

use crate::controllers::AxumRouter;
use crate::global::*;
use axum::{Extension, routing::get_service};
use http::{Method, header::HeaderValue};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use tower_cookies::CookieManagerLayer;
use tower_cookies::cookie::Key;
use tower_http::{
	cors::CorsLayer,
	services::{ServeDir, ServeFile},
};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
	// Load our evironment variables
	dotenvy::dotenv().ok();
	log::init_panic_handler();
	log::init_logger();

	// Read and store loaded environment variables
	let api_base_url = env::var("API_BASE_URL").expect("API_BASE_URL must be set");
	let front_end_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
	let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be set");

	// Initialize the database pool connection
	let pool = db::create_pool().await;

	/*
	/ Configure CORS
	/ CORS is needed when a frontend (running on one domain or port)
	/ wants to send HTTP requests to a backend running on another domain or port.
	/ This is needed for the frontend to send requests to the backend.
	/ We allow all origins, methods, and headers currently, but this should be changed later for security.
	/ TODO: Ensure we have all the right values below, may need to constrict the requests we accept
	*/
	let cors = CorsLayer::new()
		.allow_origin(
			front_end_url
				.parse::<HeaderValue>()
				.expect("Invalid frontend_url format"),
		)
		.allow_credentials(true)
		.allow_methods([Method::GET, Method::POST, Method::DELETE])
		.allow_headers([
			http::header::CONTENT_TYPE,
			http::header::ACCEPT,
			http::header::AUTHORIZATION,
			http::header::HeaderName::from_static("x-requested-with"),
		]);

	// Use an encryption/signing key for private cookies
	let cookie_key = Key::generate();

	// API routes with CORS middleware
	let api_routes = AxumRouter::new()
		.nest("/account", controllers::account::account_routes())
		.nest("/itinerary", controllers::itinerary::itinerary_routes())
		.nest("/chat", controllers::chat::chat_routes());
	// TODO: nest other routes...

	let api_routes = AxumRouter::new().nest("/api", api_routes);

	#[cfg(all(not(test), debug_assertions))]
	let api_routes = crate::swagger::merge_swagger(api_routes);

	// Build the main router
	let app = axum::Router::new()
		.merge(api_routes)
		// Static files served from /dist.
		// Fallback must be index.html since react handles routing on front end
		.fallback_service(get_service(
			ServeDir::new(DIST_DIR)
				.fallback(ServeFile::new(Path::new(DIST_DIR).join("index.html"))),
		))
		.layer(Extension(pool.clone()))
		.layer(Extension(cookie_key.clone()))
		.layer(CookieManagerLayer::new())
		.layer(cors);

	/*
	/ Bind the router to a specific port
	/ We use the SocketAddr struct to bind the router to the port
	/ We use the 0.0.0.0 address to bind the router to localhost
	/ We will bind to port 3001 for now
	*/
	let addr = SocketAddr::from_str(&bind_address).expect("Invalid BIND_ADDRESS format");
	println!("Server starting on {}", api_base_url);

	/*
	/ Serve the router ie: Start the server
	/ We will start the server with the configured router and address
	*/
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app.into_make_service()).await?;

	Ok(())
}
