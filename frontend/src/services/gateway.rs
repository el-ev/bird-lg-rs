use common::api::{AppRequest, AppResponse};

use crate::{
    store::{AppEvent, LgStateHandle},
    utils::fetch_json,
};

pub struct ApiGateway;

impl ApiGateway {
    pub fn dispatch_response(state: &LgStateHandle, response: AppResponse) {
        state.dispatch(AppEvent::ApplyResponse(response));
    }

    pub fn send_ws_request(state: &LgStateHandle, request: AppRequest) -> bool {
        if state.is_ws_connected() {
            let sender = state
                .ws_sender
                .as_ref()
                .expect("connected websocket state requires a sender");
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
