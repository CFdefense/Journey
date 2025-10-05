#![allow(unexpected_cfgs)]

// Public modules that tests can access
pub mod controllers;
pub mod db;
pub mod error;
pub mod models;
pub mod middleware;

// Public but internal modules (needed for tests and main)
pub mod global;
pub mod log;

// Re-export commonly used items
pub use error::ApiResult;
