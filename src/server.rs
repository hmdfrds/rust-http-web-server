use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::{config::Config, logger::Logger};

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

#[allow(dead_code)]
fn handle_client(mut stream: TcpStream, config: &Config, logger: &Arc<Logger>) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            // For testing: send a simple HTTP response.
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!";
            if let Err(e) = stream.write_all(response.as_bytes()) {
                logger.log_error(&format!("Error writing response: {}", e));
            } else {
                let client_ip = stream
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| "unknown".into());
                logger.log_request(&client_ip, "GET /", 200);
            }
        }
        Err(e) => {
            logger.log_error(&format!("Error reading from stream: {}", e));
        }
    }
}
