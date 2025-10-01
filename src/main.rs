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

#[cfg(not(tarpaulin_include))]
fn main() {
	dotenv::dotenv().unwrap();
	log::init_panic_handler();
	log::init_logger();

    println!("Hello, world!");
}