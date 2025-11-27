use reqwasm::http::Request;
use serde::de::DeserializeOwned;

pub type HttpResult<T> = Result<T, String>;

pub async fn fetch_json<T: DeserializeOwned>(url: &str) -> HttpResult<T> {
    match Request::get(url).send().await {
        Ok(resp) if resp.ok() => resp
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e)),
        Ok(resp) => Err(format!("HTTP request failed with status {}", resp.status())),
        Err(e) => Err(format!("Request error: {}", e)),
    }
}
