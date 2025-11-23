const DEFAULT_BACKEND_URL: &str = "http://127.0.0.1:3000";

/// Returns the backend origin retrieved from the BACKEND_URL env var or a sensible default.
pub fn backend_origin() -> &'static str {
    option_env!("BACKEND_URL").unwrap_or(DEFAULT_BACKEND_URL)
}

/// Builds a full backend URL by combining the origin with a provided path.
pub fn backend_api(path: &str) -> String {
    let origin = backend_origin().trim_end_matches('/');
    let normalized_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    format!("{}{}", origin, normalized_path)
}
