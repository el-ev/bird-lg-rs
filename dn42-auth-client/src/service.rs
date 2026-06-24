use gloo_net::http::{Request, RequestBuilder, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::models::{
    AuthMethod, AuthSessionResponse, AuthStartRequest, AuthStartResponse, HostImpersonationRequest,
    OidcCompleteRequest, OidcStartRequest, OidcStartResponse, PgpKeyLookupResponse,
    RegistryEmailCompleteRequest, RegistryEmailSendRequest, RegistryEmailSendResponse,
    RegistryEmailVerifyRequest, RegistryPgpVerifyRequest, RegistrySshVerifyRequest, UiMessage,
};

const CONFIG_PATH: &str = "/config.json";
pub const LOCALE_STORAGE_KEY: &str = "bird-lg-rs.autopeer.locale";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn current_locale_code() -> Option<String> {
    local_storage()?.get_item(LOCALE_STORAGE_KEY).ok().flatten()
}

pub fn apply_locale_header(request: RequestBuilder) -> RequestBuilder {
    if let Some(locale) = current_locale_code() {
        request.header("Accept-Language", &locale)
    } else {
        request
    }
}

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
    #[serde(default)]
    pub auth_url: Option<String>,
    #[serde(default)]
    pub allowed_return_urls: Vec<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: UiMessage,
}

pub async fn decode_json<T: DeserializeOwned>(response: Response) -> Result<T, UiMessage> {
    if response.ok() {
        response.json::<T>().await.map_err(|error| {
            UiMessage::key("error.runtime.decode_failed").with_param("detail", error.to_string())
        })
    } else {
        let status = response.status();
        match response.json::<ErrorResponse>().await {
            Ok(body) => Err(body.error),
            Err(_) => Err(UiMessage::key("error.runtime.http_failed")
                .with_param("status", status.to_string())),
        }
    }
}

pub async fn send_json<B: Serialize, T: DeserializeOwned>(
    method: &str,
    url: &str,
    token: Option<&str>,
    body: &B,
) -> Result<T, UiMessage> {
    let payload = serde_json::to_string(body).map_err(|error| {
        UiMessage::key("error.runtime.encode_failed").with_param("detail", error.to_string())
    })?;

    let request = match method {
        "POST" => Request::post(url),
        "PATCH" => Request::patch(url),
        other => {
            return Err(UiMessage::key("error.runtime.unsupported_method")
                .with_param("method", other.to_string()));
        }
    };

    let request = if let Some(token) = token {
        request.header("Authorization", &format!("Bearer {token}"))
    } else {
        request
    };
    let request = apply_locale_header(request);

    let response = request
        .header("Content-Type", "application/json")
        .body(payload)
        .map_err(|error| {
            UiMessage::key("error.runtime.request_failed").with_param("detail", error.to_string())
        })?
        .send()
        .await
        .map_err(|error| {
            UiMessage::key("error.runtime.request_failed").with_param("detail", error.to_string())
        })?;

    decode_json(response).await
}

pub async fn send_delete<T: DeserializeOwned>(url: &str, token: &str) -> Result<T, UiMessage> {
    let request = Request::delete(url).header("Authorization", &format!("Bearer {token}"));
    let request = apply_locale_header(request);
    let response = request.send().await.map_err(|error| {
        UiMessage::key("error.runtime.request_failed").with_param("detail", error.to_string())
    })?;

    decode_json(response).await
}

pub async fn send_post_empty<T: DeserializeOwned>(url: &str, token: &str) -> Result<T, UiMessage> {
    let request = Request::post(url).header("Authorization", &format!("Bearer {token}"));
    let request = apply_locale_header(request);
    let response = request.send().await.map_err(|error| {
        UiMessage::key("error.runtime.request_failed").with_param("detail", error.to_string())
    })?;

    decode_json(response).await
}

pub async fn send_get<T: DeserializeOwned>(url: &str, token: Option<&str>) -> Result<T, UiMessage> {
    let request = if let Some(token) = token {
        Request::get(url).header("Authorization", &format!("Bearer {token}"))
    } else {
        Request::get(url)
    };
    let request = apply_locale_header(request);

    let response = request.send().await.map_err(|error| {
        UiMessage::key("error.runtime.request_failed").with_param("detail", error.to_string())
    })?;

    decode_json(response).await
}

pub fn normalize_url(value: Option<String>) -> Option<String> {
    value
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}

pub fn normalize_urls(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

pub fn api_url(api_base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        api_base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub fn optional_effective_mnt(effective_mnt: Option<&str>) -> Option<String> {
    effective_mnt
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub async fn load_runtime_config() -> Result<RuntimeConfig, UiMessage> {
    let response = Request::get(CONFIG_PATH).send().await.map_err(|error| {
        UiMessage::key("error.runtime.config.load_failed").with_param("detail", error.to_string())
    })?;

    if response.status() == 404 {
        return Ok(RuntimeConfig::default());
    }

    if !response.ok() {
        return Err(UiMessage::key("error.runtime.http_failed")
            .with_param("status", response.status().to_string()));
    }

    match response.json::<RuntimeConfig>().await {
        Ok(config) => Ok(RuntimeConfig {
            autopeer_api_url: normalize_url(config.autopeer_api_url),
            autopeer_site_url: normalize_url(config.autopeer_site_url),
            looking_glass_url: normalize_url(config.looking_glass_url),
            oidc_methods: config.oidc_methods,
            auth_url: normalize_url(config.auth_url),
            allowed_return_urls: normalize_urls(config.allowed_return_urls),
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

pub async fn lookup_pgp_key(
    api_base: &str,
    fingerprint: &str,
) -> Result<PgpKeyLookupResponse, UiMessage> {
    let trimmed = fingerprint.trim();
    let path = format!("/v1/auth/lookup/pgp-key?fingerprint={trimmed}");
    let url = api_url(api_base, &path);
    send_get(&url, None).await
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
            locale: current_locale_code(),
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
    return_to: Option<&str>,
) -> Result<OidcStartResponse, UiMessage> {
    let url = api_url(api_base, &format!("/v1/auth/oidc/{provider}/start"));
    send_json(
        "POST",
        &url,
        None,
        &OidcStartRequest {
            challenge_id: challenge_id.map(str::to_string),
            return_to: return_to.map(str::to_string),
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
