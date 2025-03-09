use std::path::Path;

use chrono::Utc;

/// Preventing directory traversal.
pub fn safe_path(document_root: &str, request_path: &str) -> Result<String, String> {
    let sanitized = request_path.trim_start_matches("/");
    let joined_path = Path::new(document_root).join(sanitized);

    let canonical_join = joined_path
        .canonicalize()
        .map_err(|e| format!("Error canonicalizing path: {}", e))?;

    let canonical_doc_root = Path::new(document_root)
        .canonicalize()
        .map_err(|e| format!("Error canonicalizing document root: {}", e))?;

    if !canonical_join.starts_with(&canonical_doc_root) {
        return Err("Access denied: directory traversal attempt detected.".into());
    }

    Ok(canonical_join.to_string_lossy().into_owned())
}

/// Format a given UTC datetime into an HTTP-date string
/// Example: "Mon, 10 Mar 2024 02:46:00 GMT"
pub fn http_date_format(dt: chrono::DateTime<Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
