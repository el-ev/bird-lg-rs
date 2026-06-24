pub use dn42_auth_client::service::{
    api_url, complete_oidc, complete_registry_email, impersonate_asn, load_runtime_config,
    send_delete, send_get, send_json, send_post_empty,
};

use crate::models::{
    CreateSessionRequest, OperationStatus, SessionListResponse, UiMessage, UpdateSessionRequest,
};

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

pub async fn retire_session(
    api_base: &str,
    session_token: &str,
    node: &str,
    asn: &str,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, &format!("/v1/sessions/{node}/{asn}/retire"));
    send_post_empty(&url, session_token).await
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

pub async fn retry_operation(
    api_base: &str,
    session_token: &str,
    operation_id: &str,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, &format!("/v1/operations/{operation_id}/retry"));
    send_json("POST", &url, Some(session_token), &()).await
}

pub async fn drop_operation(
    api_base: &str,
    session_token: &str,
    operation_id: &str,
) -> Result<OperationStatus, UiMessage> {
    let url = api_url(api_base, &format!("/v1/operations/{operation_id}/drop"));
    send_json("POST", &url, Some(session_token), &()).await
}
