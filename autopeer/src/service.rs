use reqwasm::http::{Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::models::{
    AuthMethod, AuthSessionResponse, AuthStartRequest, AuthStartResponse, CreateSessionRequest,
    HostImpersonationRequest, OidcCompleteRequest, OidcStartRequest, OidcStartResponse,
    OperationStatus, RegistryEmailCompleteRequest, RegistryEmailSendRequest,
    RegistryEmailSendResponse, RegistryEmailVerifyRequest, RegistryPgpVerifyRequest,
    RegistrySshVerifyRequest, SessionListResponse, UiMessage, UpdateSessionRequest,
};

const CONFIG_PATH: &str = "/config.json";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub autopeer_api_url: Option<String>,
    #[serde(default)]
    pub autopeer_site_url: Option<String>,
    #[serde(default)]
    pub looking_glass_url: Option<String>,
    #[serde(default)]
    pub oidc_methods: Vec<AuthMethod>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: UiMessage,
}

async fn decode_json<T: DeserializeOwned>(response: Response) -> Result<T, UiMessage> {
    if response.ok() {
        response
            .json::<T>()
            .await
            .map_err(|error| UiMessage::raw(format!("Failed to decode response: {error}")))
    } else {
        let status = response.status();
        match response.json::<ErrorResponse>().await {
            Ok(body) => Err(body.error),
            Err(_) => Err(UiMessage::raw(format!(
                "HTTP request failed with status {status}"
            ))),
        }
    }
}

async fn send_json<B: Serialize, T: DeserializeOwned>(
    method: &str,
    url: &str,
    token: Option<&str>,
    body: &B,
) -> Result<T, UiMessage> {
    let payload = serde_json::to_string(body)
        .map_err(|error| UiMessage::raw(format!("Failed to encode payload: {error}")))?;

    let request = match method {
        "POST" => Request::post(url),
        "PATCH" => Request::patch(url),
        other => return Err(UiMessage::raw(format!("Unsupported HTTP method {other}"))),
    };

    let request = if let Some(token) = token {
        request.header("Authorization", &format!("Bearer {token}"))
    } else {
        request
    };

    let response = request
        .header("Content-Type", "application/json")
        .body(payload)
        .send()
        .await
        .map_err(|error| UiMessage::raw(format!("Request failed: {error}")))?;

    decode_json(response).await
}

async fn send_delete<T: DeserializeOwned>(url: &str, token: &str) -> Result<T, UiMessage> {
    let response = Request::delete(url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| UiMessage::raw(format!("Request failed: {error}")))?;

    decode_json(response).await
}

async fn send_get<T: DeserializeOwned>(url: &str, token: Option<&str>) -> Result<T, UiMessage> {
    let request = if let Some(token) = token {
        Request::get(url).header("Authorization", &format!("Bearer {token}"))
    } else {
        Request::get(url)
    };

    let response = request
        .send()
        .await
        .map_err(|error| UiMessage::raw(format!("Request failed: {error}")))?;

    decode_json(response).await
}

fn normalize_url(value: Option<String>) -> Option<String> {
    value
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}

fn api_url(api_base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        api_base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn optional_effective_mnt(effective_mnt: Option<&str>) -> Option<String> {
    effective_mnt
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub async fn load_runtime_config() -> Result<RuntimeConfig, UiMessage> {
    let response = Request::get(CONFIG_PATH)
        .send()
        .await
        .map_err(|error| UiMessage::raw(format!("Failed to load config.json: {error}")))?;

    if response.status() == 404 {
        return Ok(RuntimeConfig::default());
    }

    if !response.ok() {
        return Err(UiMessage::raw(format!(
            "Config endpoint responded with HTTP {}",
            response.status()
        )));
    }

    match response.json::<RuntimeConfig>().await {
        Ok(config) => Ok(RuntimeConfig {
            autopeer_api_url: normalize_url(config.autopeer_api_url),
            autopeer_site_url: normalize_url(config.autopeer_site_url),
            looking_glass_url: normalize_url(config.looking_glass_url),
            oidc_methods: config.oidc_methods,
        }),
        Err(_) => Ok(RuntimeConfig::default()),
    }
}

pub async fn start_auth(api_base: &str, asn: &str) -> Result<AuthStartResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/start");
    send_json(
        "POST",
        &url,
        None,
        &AuthStartRequest {
            asn: asn.to_string(),
        },
    )
    .await
}

pub async fn verify_registry_ssh(
    api_base: &str,
    challenge_id: &str,
    signature: &str,
) -> Result<AuthSessionResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/verify/registry-ssh");
    send_json(
        "POST",
        &url,
        None,
        &RegistrySshVerifyRequest {
            challenge_id: challenge_id.to_string(),
            signature: signature.to_string(),
        },
    )
    .await
}

pub async fn verify_registry_pgp(
    api_base: &str,
    challenge_id: &str,
    public_key: &str,
    signed_message: &str,
) -> Result<AuthSessionResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/verify/registry-pgp");
    send_json(
        "POST",
        &url,
        None,
        &RegistryPgpVerifyRequest {
            challenge_id: challenge_id.to_string(),
            public_key: public_key.to_string(),
            signed_message: signed_message.to_string(),
        },
    )
    .await
}

pub async fn send_registry_email(
    api_base: &str,
    challenge_id: &str,
    effective_mnt: Option<&str>,
) -> Result<RegistryEmailSendResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/verify/registry-email/send");
    send_json(
        "POST",
        &url,
        None,
        &RegistryEmailSendRequest {
            challenge_id: challenge_id.to_string(),
            effective_mnt: optional_effective_mnt(effective_mnt),
        },
    )
    .await
}

pub async fn verify_registry_email(
    api_base: &str,
    challenge_id: &str,
    code: &str,
) -> Result<AuthSessionResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/verify/registry-email");
    send_json(
        "POST",
        &url,
        None,
        &RegistryEmailVerifyRequest {
            challenge_id: challenge_id.to_string(),
            code: code.to_string(),
        },
    )
    .await
}

pub async fn complete_registry_email(
    api_base: &str,
    token: &str,
) -> Result<AuthSessionResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/verify/registry-email/complete");
    send_json(
        "POST",
        &url,
        None,
        &RegistryEmailCompleteRequest {
            token: token.to_string(),
        },
    )
    .await
}

pub async fn start_oidc(
    api_base: &str,
    provider: &str,
    challenge_id: Option<&str>,
) -> Result<OidcStartResponse, UiMessage> {
    let url = api_url(api_base, &format!("/v1/auth/oidc/{provider}/start"));
    send_json(
        "POST",
        &url,
        None,
        &OidcStartRequest {
            challenge_id: challenge_id.map(str::to_string),
        },
    )
    .await
}

pub async fn complete_oidc(api_base: &str, state: &str) -> Result<AuthSessionResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/oidc/complete");
    send_json(
        "POST",
        &url,
        None,
        &OidcCompleteRequest {
            state: state.to_string(),
        },
    )
    .await
}

pub async fn impersonate_asn(
    api_base: &str,
    session_token: &str,
    asn: &str,
    effective_mnt: Option<&str>,
) -> Result<AuthSessionResponse, UiMessage> {
    let url = api_url(api_base, "/v1/auth/impersonate");
    send_json(
        "POST",
        &url,
        Some(session_token),
        &HostImpersonationRequest {
            asn: asn.to_string(),
            effective_mnt: optional_effective_mnt(effective_mnt),
        },
    )
    .await
}

pub async fn list_sessions(
    api_base: &str,
    session_token: &str,
) -> Result<SessionListResponse, UiMessage> {
    let url = api_url(api_base, "/v1/sessions");
    send_get(&url, Some(session_token)).await
}

pub async fn create_session(
    api_base: &str,
    session_token: &str,
    request: &CreateSessionRequest,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, "/v1/sessions");
    send_json("POST", &url, Some(session_token), request).await
}

pub async fn update_session(
    api_base: &str,
    session_token: &str,
    node: &str,
    asn: &str,
    request: &UpdateSessionRequest,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, &format!("/v1/sessions/{node}/{asn}"));
    send_json("PATCH", &url, Some(session_token), request).await
}

pub async fn delete_session(
    api_base: &str,
    session_token: &str,
    node: &str,
    asn: &str,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, &format!("/v1/sessions/{node}/{asn}"));
    send_delete(&url, session_token).await
}

pub async fn get_operation(
    api_base: &str,
    session_token: &str,
    operation_id: &str,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, &format!("/v1/operations/{operation_id}"));
    send_get(&url, Some(session_token)).await
}
