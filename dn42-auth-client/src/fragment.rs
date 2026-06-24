use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::models::AuthSessionResponse;

pub fn encode_auth_session(session: &AuthSessionResponse) -> Option<String> {
    let json = serde_json::to_vec(session).ok()?;
    Some(URL_SAFE_NO_PAD.encode(json))
}

pub fn decode_auth_session(encoded: &str) -> Option<AuthSessionResponse> {
    URL_SAFE_NO_PAD
        .decode(encoded)
        .ok()
        .and_then(|decoded| serde_json::from_slice(&decoded).ok())
        .or_else(|| {
            let decoded = js_sys::decode_uri_component(encoded).ok()?.as_string()?;
            serde_json::from_str(&decoded).ok()
        })
}
