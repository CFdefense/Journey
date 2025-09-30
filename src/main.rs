mod controllers;
mod db;
mod middleware;
mod models;
mod log;
mod constants;

#[cfg(test)]
mod tests;

fn main() {
	dotenv::dotenv().unwrap();
	log::init_panic_handler();
	log::init_logger();

    println!("Hello, world!");
}
