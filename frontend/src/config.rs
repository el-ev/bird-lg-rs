use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "/config.json";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Config {
    pub username: String,
    pub backend_url: String,
}

pub async fn load_config() -> Result<Config, String> {
    fetch_config()
        .await?
        .ok_or_else(|| "No config found".into())
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
