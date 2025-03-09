use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::{
    config::Config,
    handler::{self, handle_client},
    logger::Logger,
};

/// Starts the HTTP server by binding to the configured address and accepting connections.
pub fn start_server(config: Arc<Config>, logger: Arc<Logger>) {
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).expect("Failed to bind TCP listener");
    println!("HTTP Server listening on {}", addr);
    logger.log(&format!("Server started on {}", addr));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let logger_clone = Arc::clone(&logger);
                let config_clone = Arc::clone(&config);
                // Spawn a new thread to handle the client connection.
                thread::spawn(move || {
                    handle_client(stream, &config_clone, &logger_clone);
                });
            }
            Err(e) => {
                logger.log_error(&format!("Error accepting connection: {}", e));
            }
        }
    }
}
