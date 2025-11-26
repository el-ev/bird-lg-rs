use reqwasm::http::Request;
use serde::de::DeserializeOwned;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::ReadableStreamDefaultReader;

use crate::utils::js_value_to_string;

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

pub async fn stream_fetch<F>(url: String, mut on_chunk: F) -> Result<(), String>
where
    F: FnMut(String),
{
    let window = web_sys::window().ok_or_else(|| "Browser window unavailable".to_string())?;
    let response_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(js_value_to_string)?;
    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| "Failed to cast fetch response".to_string())?;

    if !response.ok() {
        let status = response.status();
        let error_body = match response.text() {
            Ok(promise) => match JsFuture::from(promise).await {
                Ok(value) => value.as_string().unwrap_or_default(),
                Err(e) => js_value_to_string(e),
            },
            Err(e) => js_value_to_string(e),
        };

        let trimmed = error_body.trim();
        let message = if trimmed.is_empty() {
            format!("Request failed with HTTP {}", status)
        } else {
            trimmed.to_string()
        };

        return Err(message);
    }

    let body = response
        .body()
        .ok_or_else(|| "Response body was empty".to_string())?;
    let reader: ReadableStreamDefaultReader = body
        .get_reader()
        .dyn_into()
        .map_err(|_| "Streaming reader unsupported".to_string())?;

    loop {
        let chunk = JsFuture::from(reader.read())
            .await
            .map_err(js_value_to_string)?;
        let done = js_sys::Reflect::get(&chunk, &JsValue::from_str("done"))
            .map_err(js_value_to_string)?
            .as_bool()
            .unwrap_or(false);
        if done {
            break;
        }

        let value = js_sys::Reflect::get(&chunk, &JsValue::from_str("value"))
            .map_err(js_value_to_string)?;
        let bytes = js_sys::Uint8Array::new(&value).to_vec();
        let text = String::from_utf8(bytes)
            .map_err(|_| "Response chunk was not valid UTF-8".to_string())?;
        on_chunk(text);
    }

    Ok(())
}
