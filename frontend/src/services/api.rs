use crate::store::modal::ModalAction;
use crate::store::traceroute::TracerouteAction;
use crate::store::{Action, NodeTracerouteResult};
use crate::utils::fetch_json;
use common::api::{AppRequest, AppResponse};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

pub fn perform_traceroute(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    version: String,
) {
    let state = state.clone();

    spawn_local(async move {
        state.dispatch(Action::Traceroute(TracerouteAction::InitResult(
            node.clone(),
        )));

        if let Some(sender) = &state.ws_sender {
            sender.emit(AppRequest::Traceroute {
                node: node.clone(),
                target: target.clone(),
                version: version.clone(),
            });
        } else {
            let version_param = match version.as_str() {
                "4" => "&version=4",
                "6" => "&version=6",
                _ => "",
            };

            let url = format!(
                "{}/api/traceroute?node={}&target={}{}",
                state.backend_url, node, target, version_param
            );

            let state_clone = state.clone();
            let node_name = node.clone();

            spawn_local(async move {
                match fetch_json::<AppResponse>(&url).await {
                    Ok(response) => {
                        if let AppResponse::Error(err) = &response {
                            state_clone.dispatch(Action::Traceroute(
                                TracerouteAction::UpdateResult(
                                    node_name,
                                    NodeTracerouteResult::Error(err.clone()),
                                ),
                            ));
                        } else {
                            crate::services::response_handler::handle_app_response(
                                response,
                                &state_clone,
                            );
                        }
                    }
                    Err(err) => {
                        tracing::error!("Traceroute failed for {}: {}", node_name, err);
                        state_clone.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                            node_name,
                            NodeTracerouteResult::Error(err.to_string()),
                        )));
                    }
                }
            });
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

    state.dispatch(Action::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(command),
    }));

    if let Some(sender) = &state.ws_sender {
        sender.emit(AppRequest::RouteLookup { node, target, all });
    } else {
        spawn_local(async move {
            let url = format!(
                "{}/api/routes/{}?target={}&all={}",
                state.backend_url, node, target, all
            );

            match fetch_json::<AppResponse>(&url).await {
                Ok(response) => {
                    if let AppResponse::Error(err) = &response {
                        state.dispatch(Action::Modal(ModalAction::UpdateContent(format!(
                            "Error: {}",
                            err
                        ))));
                    } else {
                        crate::services::response_handler::handle_app_response(response, &state);
                    }
                }
                Err(err) => {
                    state.dispatch(Action::Modal(ModalAction::UpdateContent(format!(
                        "Failed to load route details: {}",
                        err
                    ))));
                }
            }
        });
    }
}

pub async fn get_protocols(state: &UseReducerHandle<crate::store::LgState>) -> Result<(), String> {
    let url = format!("{}/api/protocols", state.backend_url.trim_end_matches('/'));
    match fetch_json::<AppResponse>(&url).await {
        Ok(response) => {
            if let AppResponse::Error(e) = response {
                Err(e)
            } else {
                crate::services::response_handler::handle_app_response(response, state);
                Ok(())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn get_network_info(
    state: &UseReducerHandle<crate::store::LgState>,
) -> Result<(), String> {
    let url = format!("{}/api/info", state.backend_url.trim_end_matches('/'));
    match fetch_json::<AppResponse>(&url).await {
        Ok(response) => {
            if let AppResponse::Error(e) = response {
                Err(e)
            } else {
                crate::services::response_handler::handle_app_response(response, state);
                Ok(())
            }
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

    state.dispatch(Action::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(format!(
            "{}@{}$ birdc show protocols all {}",
            state.username, node, proto
        )),
    }));

    if let Some(sender) = &state.ws_sender {
        sender.emit(AppRequest::ProtocolDetails {
            node,
            protocol: proto,
        });
    } else {
        spawn_local(async move {
            let url = format!("{}/api/protocols/{}/{}", state.backend_url, node, proto);

            match fetch_json::<AppResponse>(&url).await {
                Ok(response) => {
                    if let AppResponse::Error(err) = &response {
                        state.dispatch(Action::Modal(ModalAction::UpdateContent(format!(
                            "Error: {}",
                            err
                        ))));
                    } else {
                        crate::services::response_handler::handle_app_response(response, &state);
                    }
                }
                Err(err) => {
                    state.dispatch(Action::Modal(ModalAction::UpdateContent(format!(
                        "Failed to load protocol details: {}",
                        err
                    ))));
                }
            }
        });
    }
}

pub fn request_wireguard(state: &UseReducerHandle<crate::store::LgState>) {
    if let Some(sender) = &state.ws_sender {
        sender.emit(AppRequest::GetWireGuard);
    }
}
