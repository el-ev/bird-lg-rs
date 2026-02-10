use common::api::{AppRequest, AppResponse};

use crate::{store::LgStateHandle, utils::fetch_json};

pub struct ApiGateway;

impl ApiGateway {
    pub fn dispatch_response(state: &LgStateHandle, response: AppResponse) {
        crate::services::response_handler::handle_app_response(response, state);
    }

    pub async fn send_or_fetch(
        state: &LgStateHandle,
        request: AppRequest,
        fallback_url: Option<String>,
    ) -> Result<Option<AppResponse>, String> {
        if let Some(sender) = &state.ws_sender {
            sender.emit(request);
            return Ok(None);
        }

        let Some(url) = fallback_url else {
            return Err("WebSocket unavailable and no HTTP fallback endpoint".to_string());
        };

        let response = fetch_json::<AppResponse>(&url).await?;
        Ok(Some(response))
    }

    pub async fn fetch_response(url: String) -> Result<AppResponse, String> {
        fetch_json::<AppResponse>(&url).await
    }
}
