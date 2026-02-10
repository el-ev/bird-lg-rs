use common::api::{AppRequest, AppResponse};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    services::gateway::ApiGateway,
    store::{
        AppEvent, ProtocolDetailsContext, RouteLookupContext, TracerouteResult, modal::ModalAction,
        ping::{PingAction, PingResult},
        traceroute::TracerouteAction,
    },
};

pub fn perform_traceroute(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    version: String,
) {
    let state = state.clone();

    spawn_local(async move {
        state.dispatch(AppEvent::Traceroute(TracerouteAction::InitResult(
            node.clone(),
        )));

        let version_param = match version.as_str() {
            "4" => "&version=4",
            "6" => "&version=6",
            _ => "",
        };
        let url = format!(
            "{}/api/traceroute?node={}&target={}{}",
            state.backend_url.trim_end_matches('/'),
            node,
            target,
            version_param
        );

        match ApiGateway::send_or_fetch(
            &state,
            AppRequest::Traceroute {
                node: node.clone(),
                target: target.clone(),
                version: version.clone(),
            },
            Some(url),
        )
        .await
        {
            Ok(Some(AppResponse::Error(err))) => {
                state.dispatch(AppEvent::Traceroute(TracerouteAction::UpdateResult(
                    node,
                    TracerouteResult::Error(err),
                )));
            }
            Ok(Some(response)) => ApiGateway::dispatch_response(&state, response),
            Ok(None) => {}
            Err(err) => {
                tracing::error!("Traceroute failed for {}: {}", node, err);
                state.dispatch(AppEvent::Traceroute(TracerouteAction::UpdateResult(
                    node,
                    TracerouteResult::Error(err),
                )));
            }
        }
    });
}

pub fn perform_ping(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    version: String,
) {
    state.dispatch(AppEvent::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(format!(
            "{}@{}$ ping -c 5 {} {}",
            state.username, node, version, target
        )),
    }));
    let state = state.clone();
    spawn_local(async move {
        match ApiGateway::send_or_fetch(
            &state,
            AppRequest::Ping {
                node: node.clone(),
                target,
                version,
            },
            None,
        )
        .await
        {
            Ok(Some(response)) => ApiGateway::dispatch_response(&state, response),
            Ok(None) => {}
            Err(err) => {
                state.dispatch(AppEvent::Ping(PingAction::SetError(err.clone())));
                state.dispatch(AppEvent::PingModalUpdate {
                    node,
                    result: PingResult::Error(err),
                });
            }
        }
    });
}

pub fn perform_route_lookup(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    all: bool,
) {
    let state = state.clone();

    let command = if all {
        format!(
            "{}@{}$ birdc show route {} all",
            state.username, node, target
        )
    } else {
        format!("{}@{}$ birdc show route {}", state.username, node, target)
    };

    state.dispatch(AppEvent::SetRouteLookupContext(RouteLookupContext {
        node: node.clone(),
        target: target.clone(),
        all,
    }));
    state.dispatch(AppEvent::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(command),
    }));

    spawn_local(async move {
        let url = format!(
            "{}/api/routes/{}?target={}&all={}",
            state.backend_url.trim_end_matches('/'),
            node,
            target,
            all
        );

        match ApiGateway::send_or_fetch(
            &state,
            AppRequest::RouteLookup {
                node: node.clone(),
                target,
                all,
            },
            Some(url),
        )
        .await
        {
            Ok(Some(AppResponse::Error(err))) => {
                state.dispatch(AppEvent::Modal(ModalAction::UpdateContent(format!(
                    "Error: {}",
                    err
                ))));
            }
            Ok(Some(response)) => ApiGateway::dispatch_response(&state, response),
            Ok(None) => {}
            Err(err) => {
                state.dispatch(AppEvent::Modal(ModalAction::UpdateContent(format!(
                    "Failed to load route details: {}",
                    err
                ))));
            }
        }
    });
}

pub async fn get_protocols(state: &UseReducerHandle<crate::store::LgState>) -> Result<(), String> {
    let url = format!("{}/api/protocols", state.backend_url.trim_end_matches('/'));
    match ApiGateway::send_or_fetch(state, AppRequest::GetProtocols, Some(url)).await {
        Ok(Some(AppResponse::Error(e))) => Err(e),
        Ok(Some(response)) => {
            ApiGateway::dispatch_response(state, response);
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn get_network_info(
    state: &UseReducerHandle<crate::store::LgState>,
) -> Result<(), String> {
    let url = format!("{}/api/info", state.backend_url.trim_end_matches('/'));
    match ApiGateway::fetch_response(url).await {
        Ok(AppResponse::Error(e)) => Err(e),
        Ok(response) => {
            ApiGateway::dispatch_response(state, response);
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_protocol_details(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    proto: String,
) {
    let state = state.clone();

    state.dispatch(AppEvent::SetProtocolDetailsContext(ProtocolDetailsContext {
        node: node.clone(),
        protocol: proto.clone(),
    }));
    state.dispatch(AppEvent::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(format!(
            "{}@{}$ birdc show protocols all {}",
            state.username, node, proto
        )),
    }));

    spawn_local(async move {
        let url = format!(
            "{}/api/protocols/{}/{}",
            state.backend_url.trim_end_matches('/'),
            node,
            proto
        );

        match ApiGateway::send_or_fetch(
            &state,
            AppRequest::ProtocolDetails {
                node: node.clone(),
                protocol: proto,
            },
            Some(url),
        )
        .await
        {
            Ok(Some(AppResponse::Error(err))) => {
                state.dispatch(AppEvent::Modal(ModalAction::UpdateContent(format!(
                    "Error: {}",
                    err
                ))));
            }
            Ok(Some(response)) => ApiGateway::dispatch_response(&state, response),
            Ok(None) => {}
            Err(err) => {
                state.dispatch(AppEvent::Modal(ModalAction::UpdateContent(format!(
                    "Failed to load protocol details: {}",
                    err
                ))));
            }
        }
    });
}

pub fn request_wireguard(state: &UseReducerHandle<crate::store::LgState>) {
    let state = state.clone();
    spawn_local(async move {
        if let Err(err) = ApiGateway::send_or_fetch(&state, AppRequest::GetWireGuard, None).await {
            state.dispatch(AppEvent::SetError(format!("WireGuard refresh failed: {}", err)));
        }
    });
}
