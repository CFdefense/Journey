mod controllers;
mod db;
mod global;
mod log;
mod middleware;
mod models;
mod error;

#[cfg(test)]
mod test;

fn main() {
    dotenv::dotenv().unwrap();
    log::init_panic_handler();
    log::init_logger();

    println!("Hello, world!");
}
