use std::{process, sync::Arc, time::Duration};

use admin::AdminInterface;
use config::Config;
use logger::Logger;

mod admin;
mod config;
mod handler;
mod logger;
mod server;
mod utils;

fn main() {
    // Load configuration
    let config = Arc::new(Config::load_from_file("config.json").unwrap_or_else(|e| {
        eprintln!("Error loading configuration: {}", e);
        process::exit(1);
    }));

    println!("Configuration loaded: {:?}", config);

    // Initialize logger and wrap it in an Arc for shared ownership
    let logger = Arc::new(Logger::new(&config.log_file));
    logger.clone().start_periodic_stats(Duration::from_secs(60));

    // Start the HTTP server in a separate thread
    let logger_clone = Arc::clone(&logger);
    let config_clone = config.clone();
    std::thread::spawn(move || {
        server::start_server(config_clone, logger_clone);
    });

    // Start the admin interface.
    let admin_interface = AdminInterface::new(&config.host, config.admin_port, Arc::clone(&logger));
    admin_interface.start();

    // Prevent main from exiting immediately.
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
