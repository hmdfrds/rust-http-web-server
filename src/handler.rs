use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;

use chrono::Utc;
use chrono::format;
use mime_guess::from_path;

use crate::Config;
use crate::Logger;
use crate::utils::http_date_format;
use crate::utils::safe_path;

/// Handles an individual HTTP request on the given TCP stream.
pub fn handle_client(mut stream: TcpStream, config: &Config, logger: &Logger) {
    let peer_addr = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();

    // Read the request line
    if let Err(e) = reader.read_line(&mut request_line) {
        logger.log_error(&format!("Error reading request line: {}", e));
        return;
    }

    let request_line = request_line.trim_end();
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        send_response(
            &mut stream,
            400,
            "Bad Request",
            "Invalid request line",
            None,
        );
        return;
    }
    let method = parts[0];
    let raw_path = parts[1];
    let _version = parts[2];

    // Read and discard HTTP headers until an empty line is encountered.
    loop {
        let mut header_line = String::new();
        match reader.read_line(&mut header_line) {
            Ok(0) => break,
            Ok(_) => {
                if header_line.trim().is_empty() {
                    break;
                }
            }
            Err(e) => {
                logger.log_error(&format!("Error reading headers: {}", e));
                break;
            }
        }
    }

    let file_path = match safe_path(&config.document_root, raw_path) {
        Ok(path) => path,
        Err(e) => {
            send_response(&mut stream, 403, "Forbidden", "Access denied", None);
            logger.log_error(&format!("Forbidden access: {}", e));
            return;
        }
    };

    // Check if the file or directory exists.
    let metadata = fs::metadata(&file_path);
    if metadata.is_err() {
        send_response(
            &mut stream,
            404,
            "Not Found",
            "<html><body><h1>404 Not Found</h1></body></html>",
            None,
        );
        logger.log_request(&peer_addr, request_line, 404);
        return;
    }
    let metadata = metadata.unwrap();

    // If the path is a directory, try to serve index.html or generating a listing.
    if metadata.is_dir() {
        let index_path = Path::new(&file_path).join("index.html");
        if index_path.exists() {
            serve_file(
                &mut stream,
                &index_path,
                method,
                request_line,
                logger,
                &peer_addr,
            );
        } else {
            let listing = generate_directory_listing(Path::new(&file_path));
            let headers = Some(vec![
                ("Content-Type".into(), "text/html".into()),
                ("Content-Length".into(), listing.len().to_string()),
                ("Date".into(), http_date_format(Utc::now())),
                ("Connection".into(), "close".into()),
            ]);
            send_response(&mut stream, 200, "OK", &listing, headers);
            logger.log_request(&peer_addr, request_line, 200);
        }
        return;
    }
    // If it's a file, serve it.
    serve_file(
        &mut stream,
        &PathBuf::from(&file_path),
        method,
        request_line,
        logger,
        &peer_addr,
    );
}

/// Server a static file to the client.
fn serve_file(
    stream: &mut TcpStream,
    file_path: &Path,
    method: &str,
    request_line: &str,
    logger: &Logger,
    peer_addr: &str,
) {
    let data = fs::read(file_path);
    if let Err(e) = data {
        send_response(
            stream,
            500,
            "Internal Server Error",
            "<html><body><h1>500 Internal Server Error</h1></body></html>",
            None,
        );
        logger.log_error(&format!("Error reading file: {}", e));
        return;
    }
    let data = data.unwrap();
    let mime_type = from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    let headers = Some(vec![
        ("Content-Type".into(), mime_type),
        ("Content-Length".into(), data.len().to_string()),
        ("Date".into(), http_date_format(Utc::now())),
        ("Server".into(), "RustHTTP/1.0".into()),
        ("Connection".into(), "close".into()),
    ]);

    // For HEAD requests, send only headers.
    if method.eq_ignore_ascii_case("HEAD") {
        send_response(stream, 200, "OK", "", headers);
    } else {
        send_response_with_body(stream, 200, "OK", &data, headers);
    }
    logger.log_request(peer_addr, request_line, 200);
}

/// Sends an HTTP response without a binary body.
fn send_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    body: &str,
    headers: Option<Vec<(String, String)>>,
) {
    let mut response = format!("HTTP/1.1 {} {}\r\n", status_code, status_code);
    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
    }
    response.push_str("\r\n");
    response.push_str(body);
    let _ = stream.write_all(response.as_bytes());
}

/// Sends an HtTP response with a binary body.
fn send_response_with_body(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    body: &[u8],
    headers: Option<Vec<(String, String)>>,
) {
    let mut header_text = format!("HTTP/1.1 {} {}\r\n", status_code, status_text);
    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            header_text.push_str(&format!("{}: {}\r\n", key, value));
        }
    }
    header_text.push_str("\r\n");
    let _ = stream.write_all(header_text.as_bytes());
    let _ = stream.write_all(body);
}

/// Generates a simple HTML directory listing for the give directory.
fn generate_directory_listing(dir_path: &Path) -> String {
    let mut listing = format!(
        "<html><head><title>Directory Listing for {}</title></head><body>",
        dir_path.display()
    );
    listing.push_str(&format!(
        "<h1>Directory listing for {}</h1><ul>",
        dir_path.display()
    ));
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let file_name = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| "unknown".into());
            listing.push_str(&format!("<li>{}</li>", file_name));
        }
    }
    listing.push_str("</ul></body></html>");
    listing
}
