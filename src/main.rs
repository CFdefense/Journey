#![allow(unexpected_cfgs)]

mod controllers;
mod db;
mod global;
mod log;
mod middleware;
mod models;
mod error;

use axum::{Router, Extension};
use http::{Method, header::HeaderValue};
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::{
	cors::CorsLayer,
	services::ServeDir
};
use tower_cookies::CookieManagerLayer;

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
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
            http::header::HeaderName::from_static("x-requested-with"),
        ]);

    // Import routes (attach shared state and cookies)
    let account_routes = controllers::account::account_routes()
        .layer(Extension(pool.clone()))
        .layer(CookieManagerLayer::new());
    // TODO: Add More...

    // TODO: Intialize cookies

    // API routes with CORS middleware
    let api_routes = Router::new()
    	.nest("/account", account_routes)
     	// TODO: nest other routes (like itinerary and stuff)
        .layer(cors);

    // Build the main router
    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("./frontend/dist"));

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
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
