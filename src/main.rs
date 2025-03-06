use std::process;

use config::Config;

mod admin;
mod config;
mod handler;
mod logger;
mod utils;

fn main() {
    // Load configuration
    let config = match Config::load_from_file("config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            process::exit(1)
        }
    };

    println!("Configuration loaded successfully: {:?}", config);
}
