use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use base64::Engine;

use crate::logger::Logger;

/// AdminInterface holds configuration for the admin server.
pub struct AdminInterface {
    pub host: String,
    pub admin_port: u16,
    pub username: String,
    pub password: String,
    pub logger: Arc<Logger>,
}

impl AdminInterface {
    /// Creates a new AdminInterface with hardcoded credentials.
    pub fn new(host: &str, admin_port: u16, logger: Arc<Logger>) -> Self {
        AdminInterface {
            host: host.to_string(),
            admin_port,
            username: "admin".to_string(),
            password: "adminpass".to_string(),
            logger,
        }
    }

    /// Starts the admin server in a new thread.
    pub fn start(&self) {
        let addr = format!("{}:{}", self.host, self.admin_port);
        let listener = TcpListener::bind(&addr).expect("Failed to bind admin interface");
        println!("Admin interface listening on {}", addr);

        let logger = Arc::clone(&self.logger);
        let username = self.username.clone();
        let password = self.password.clone();

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let logger = Arc::clone(&logger);
                    let username = username.clone();
                    let password = password.clone();
                    thread::spawn(move || {
                        handle_admin_request(stream, &username, &password, &logger);
                    });
                }
            }
        });
    }
}

/// Handles an individual admin request.
fn handle_admin_request(
    mut stream: TcpStream,
    username: &str,
    password: &str,
    logger: &Arc<Logger>,
) {
    let mut buffer = [0; 1024];
    // Read the request (for a robust solution, read until the header is fully received)
    let _ = stream.read(&mut buffer);
    let request = String::from_utf8_lossy(&buffer);

    // Look for the Authorization header.
    let auth_header = request
        .lines()
        .find(|line| line.starts_with("Authorization: Basic "));

    if let Some(header) = auth_header {
        // Extract the encoded part.
        let encoded = header.trim_start_matches("Authorization: Basic ").trim();
        match base64::engine::general_purpose::STANDARD.decode(encoded) {
            Ok(decoded_bytes) => {
                if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                    // Check if the decoded credentials match "username:password"
                    if decoded_str != format!("{}:{}", username, password) {
                        let response = "HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"Admin Interface\"\r\nContent-Length: 0\r\n\r\n";
                        let _ = stream.write_all(response.as_bytes());
                        return;
                    }
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                    return;
                }
            }
            Err(_) => {
                let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
                let _ = stream.write_all(response.as_bytes());
                return;
            }
        }
    } else {
        // No Authorization header found.
        let response = "HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"Admin Interface\"\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    // Generate server statistics.
    let stats = format!(
        "Total Request: {}\nUptime: {} seconds\n",
        logger.total_requests(),
        logger.uptime()
    );
    // Read the last 10 log entries from the log file.
    let log_content = fs::read_to_string(&logger.log_file_path())
        .unwrap_or_else(|_| "No logs available".to_string());
    let log_lines: Vec<&str> = log_content.lines().rev().take(10).collect();
    let logs = log_lines.join("\n");

    // Build the admin HTML page with a meta refresh every 30 seconds.
    let html = format!(
        "<html><head><meta http-equiv=\"refresh\" content=\"30\"><title>Admin Interface</title></head>\
         <body><h1>Admin Interface</h1><pre>{}\n\nLast 10 Log Entries:\n{}</pre></body></html>",
        stats, logs
    );

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );

    let _ = stream.write_all(response.as_bytes());
}
