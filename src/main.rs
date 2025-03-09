use std::{process, sync::Arc, time::Duration};

use config::Config;
use logger::Logger;

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

    // Initialize the logger
    let logger = Arc::new(Logger::new(&config.log_file));
    // Start periodic logging every 60 seconds
    logger.clone().start_periodic_stats(Duration::from_secs(60));

    // Example log entries for testing
    logger.log_request("127.0.0.1", "GET /index.html HTTP/1.1", 200);
    logger.log_error("Test error message");

    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
