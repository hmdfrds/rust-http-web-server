use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub admin_port: u16,
    pub document_root: String,
    pub max_threads: usize,
    pub log_file: String,
}

impl Config {
    /// Loads configuration from a JSON file and validates required fields.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
        // Read the file content into a string.
        let contents = fs::read_to_string(path)?;
        // Deserialize the JSON into our Config struct.
        let config: Config = serde_json::from_str(&contents)?;

        // Basic validation
        if config.host.is_empty() {
            return Err("host cannot be empty".into());
        }
        if config.port == 0 {
            return Err("port must be a non-zero value".into());
        }
        if config.admin_port == 0 {
            return Err("admin_port must be a non-zero value".into());
        }
        if config.document_root.is_empty() {
            return Err("document_root cannot be empty".into());
        }
        if config.log_file.is_empty() {
            return Err("log_file cannot be empty".into());
        }
        Ok(config)
    }
}
