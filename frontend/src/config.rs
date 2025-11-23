use reqwasm::http::Request;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::services::log_info;

const DEFAULT_BACKEND_URL: &str = "http://127.0.0.1:3000";
const CONFIG_PATH: &str = "/config.json";

static BACKEND_ORIGIN: OnceLock<String> = OnceLock::new();

#[derive(Deserialize)]
struct RuntimeConfig {
    backend_url: String,
}

pub async fn load_backend_origin() -> Result<&'static str, String> {
    if let Some(origin) = backend_origin() {
        return Ok(origin);
    }

    if let Some(origin) = fetch_runtime_backend_origin().await?
        && let Some(stored) = try_store_backend_origin(origin)
    {
        return Ok(stored);
    }
    if let Some(env_origin) = option_env!("BACKEND_URL")
        && let Some(stored) = try_store_backend_origin(env_origin)
    {
        return Ok(stored);
    }

    Ok(try_store_backend_origin(DEFAULT_BACKEND_URL).unwrap())
}

fn try_store_backend_origin<S: AsRef<str>>(origin: S) -> Option<&'static str> {
    let normalized = origin.as_ref().trim().trim_end_matches('/').to_string();

    if !normalized.is_empty() {
        let _ = BACKEND_ORIGIN.set(normalized.clone());
        log_info(&format!("Backend: {normalized}"));
        return BACKEND_ORIGIN.get().map(|s| s.as_str());
    }

    None
}

async fn fetch_runtime_backend_origin() -> Result<Option<String>, String> {
    let response = Request::get(CONFIG_PATH)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status() == 404 {
        return Ok(None);
    }
    if !response.ok() {
        return Err(format!(
            "Config endpoint responded with HTTP {}",
            response.status()
        ));
    }

    let config = response
        .json::<RuntimeConfig>()
        .await
        .map_err(|e| e.to_string())?;
    let trimmed = config.backend_url.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.trim_end_matches('/').to_string()))
}

pub fn backend_origin() -> Option<&'static str> {
    BACKEND_ORIGIN.get().map(|s| s.as_str())
}

pub fn backend_api(path: &str) -> String {
    let origin = backend_origin().expect("No valid backend origin configured");
    let path = if path.starts_with('/') { path } else { "/" };
    format!("{}{}", origin.trim_end_matches('/'), path)
}
