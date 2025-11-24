use reqwasm::http::Request;
use serde::Deserialize;
use std::sync::OnceLock;

const CONFIG_PATH: &str = "/config.json";

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Deserialize)]
pub struct Config {
    username: String,
    backend_url: String,
}

pub async fn load_config() -> Result<&'static Config, String> {
    if let Some(c) = config() {
        return Ok(c);
    }

    if let Some(origin) = fetch_config().await? {
        return Ok(CONFIG.get_or_init(|| origin));
    }

    Err("No config found".into())
}

async fn fetch_config() -> Result<Option<Config>, String> {
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

    let config = response.json::<Config>().await.map_err(|e| e.to_string())?;
    Ok(Some(config))
}

fn config() -> Option<&'static Config> {
    CONFIG.get()
}

pub fn backend_api(path: &str) -> String {
    let config = config().expect("No runtime config found");
    let path = if path.starts_with('/') { path } else { "/" };
    format!("{}{}", config.backend_url.trim_end_matches('/'), path)
}

pub fn username() -> String {
    let config = config().expect("No runtime config found");
    config.username.clone()
}
