mod http;
pub use http::*;

use wasm_bindgen::JsValue;
use web_sys::console;

pub async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

pub fn log_error(message: &str) {
    console::error_1(&JsValue::from_str(message));
}
