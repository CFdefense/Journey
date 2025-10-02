#![allow(unexpected_cfgs)]

mod controllers;
mod db;
mod middleware;
mod models;
mod log;
mod constants;
mod error;

#[cfg(test)]
mod test;

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use axum::Router;
use tower_http::cors::CorsLayer;
use http::{Method, header::HeaderValue};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load our evironment variables
    dotenv::dotenv().ok();

    // Read and store loaded environment variables
    let api_base_url = env::var("API_BASE_URL").expect("API_BASE_URL must be set");
    let front_end_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be set");

    // Initialize the database pool connection (TODO: use in extensions)
    let _pool = db::create_pool();

    /*
    / Configure CORS
    / CORS is needed when a frontend (running on one domain or port)
    / wants to send HTTP requests to a backend running on another domain or port.
    / This is needed for the frontend to send requests to the backend.
    / We allow all origins, methods, and headers currently, but this should be changed later for security.
    / TODO: Ensure we have all the right values below, may need to constrict the requests we accept
    */
    let cors = CorsLayer::new()
        .allow_origin(front_end_url.parse::<HeaderValue>().expect("Invalid frontend_url format"))
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
            http::header::HeaderName::from_static("x-requested-with"),
        ]);

    // Import routes
    let account_routes = controllers::account::account_routes();
    // TODO: Add More...

    // TODO: Intialize cookies

    // Build the main router with CORS middleware
    let app = Router::new()
        .nest("/account", account_routes)
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
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}