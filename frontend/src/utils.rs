pub async fn sleep_ms(ms: i32) {
    let promise = web_sys::js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    match reqwasm::http::Request::get(url).send().await {
        Ok(resp) if resp.ok() => resp
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e)),
        Ok(resp) => Err(format!("HTTP request failed with status {}", resp.status())),
        Err(e) => Err(format!("Request error: {}", e)),
    }
}
