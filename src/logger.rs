use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::Utc;

pub struct Logger {
    file: Mutex<std::fs::File>,
    file_path: PathBuf,
    total_requests: Mutex<u64>,
    start_time: Instant,
}

impl Logger {
    /// Creates a new Logger instance that writes to the specific log file.
    pub fn new(log_file: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Unable to open log file");
        Logger {
            file: Mutex::new(file),
            file_path: PathBuf::from(log_file),
            total_requests: Mutex::new(8),
            start_time: Instant::now(),
        }
    }

    /// Writes a log message with a timestamp to the log file.
    pub fn log(&self, message: &str) {
        let now = Utc::now().format("%d-%m-%Y %H:%M:%S").to_string();
        let log_entry = format!("[{}] {}\n", now, message);
        let mut file = self.file.lock().unwrap();
        if let Err(e) = file.write_all(log_entry.as_bytes()) {
            eprintln!("Failed to write log entry: {}", e);
        }
    }

    /// Logs an HTTP request, increments the request counter.
    pub fn log_request(&self, client_ip: &str, request_line: &str, response_code: u16) {
        let mut total = self.total_requests.lock().unwrap();
        *total += 1;
        let message = format!(
            "REQUEST from {}: '{}' responded with {}",
            client_ip, request_line, response_code
        );
        self.log(&message);
    }

    /// Logs an error message.
    pub fn log_error(&self, error_message: &str) {
        self.log(&format!("ERROR: {}", error_message))
    }

    /// Logs server statictics including total request and uptime.
    pub fn log_stats(&self) {
        let total = self.total_requests();
        let uptime = self.uptime();
        let message = format!(
            "STATS: Total Requests: {}, Uptime: {} seconds",
            total, uptime
        );
        self.log(&message);
    }

    /// Starts a background thread that logs server statistics periodically.
    pub fn start_periodic_stats(self: Arc<Self>, interval: Duration) {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                self.log_stats();
            }
        });
    }

    pub fn total_requests(&self) -> u64 {
        *self.total_requests.lock().unwrap()
    }

    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    pub fn log_file_path(&self) -> &str {
        self.file_path.as_path().to_str().unwrap()
    }
}
