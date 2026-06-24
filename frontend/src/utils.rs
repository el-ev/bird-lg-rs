use common::models::NodeProtocol;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, window};
use yew::MouseEvent;

pub async fn sleep_ms(ms: i32) {
    let promise = web_sys::js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    match gloo_net::http::Request::get(url).send().await {
        Ok(resp) if resp.ok() => resp
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e)),
        Ok(resp) => Err(format!("HTTP request failed with status {}", resp.status())),
        Err(e) => Err(format!("Request error: {}", e)),
    }
}

pub fn select_text(e: MouseEvent) {
    if let Some(target) = e.target()
        && let Ok(element) = target.dyn_into::<HtmlElement>()
        && let Some(window) = window()
        && let Ok(Some(selection)) = window.get_selection()
    {
        let _ = selection.remove_all_ranges();
        if let Some(document) = window.document()
            && let Ok(range) = document.create_range()
            && range.select_node_contents(&element).is_ok()
        {
            let _ = selection.add_range(&range);
        }
    }
}

pub fn get_hostname() -> String {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn is_dn42_domain() -> bool {
    get_hostname().ends_with(".dn42")
}

pub fn get_hash_route() -> Option<String> {
    let hash = window()?.location().hash().ok()?;
    let trimmed = hash.strip_prefix('#').unwrap_or(&hash);
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn set_hash_route(hash: &str) {
    if let Some(window) = window()
        && let Ok(history) = window.history()
    {
        let url = format!("#{hash}");
        let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&url));
    }
}

pub fn clear_hash_route() {
    if let Some(window) = window()
        && let Ok(history) = window.history()
    {
        let pathname = window.location().pathname().unwrap_or_default();
        let search = window.location().search().unwrap_or_default();
        let _ = history.replace_state_with_url(
            &JsValue::NULL,
            "",
            Some(&format!("{pathname}{search}")),
        );
    }
}

pub fn resolve_hash_protocol(
    node_name: Option<&str>,
    nodes: &[NodeProtocol],
) -> Option<(String, String)> {
    let hash = get_hash_route()?;
    let (target_node, target_proto) = if let Some(node) = node_name {
        (node.to_string(), hash)
    } else {
        let (n, p) = hash.split_once('/')?;
        (n.to_string(), p.to_string())
    };

    let node = nodes.iter().find(|n| n.name == target_node)?;
    node.protocols.iter().find(|p| p.name == target_proto)?;

    Some((target_node, target_proto))
}
