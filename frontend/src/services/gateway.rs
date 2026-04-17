use common::api::{AppRequest, AppResponse};

use crate::{store::LgStateHandle, utils::fetch_json};

pub struct ApiGateway;

impl ApiGateway {
    pub fn dispatch_response(state: &LgStateHandle, response: AppResponse) {
        crate::services::response_handler::handle_app_response(response, state);
    }

    pub fn send_ws_request(state: &LgStateHandle, request: AppRequest) -> bool {
        if let Some(sender) = &state.ws_sender {
            sender.emit(request);
            true
        } else {
            false
        }
    }

    pub async fn fetch_response(url: String) -> Result<AppResponse, String> {
        fetch_json::<AppResponse>(&url).await
    }
}
