use common::api::{AppRequest, AppResponse};
use yew::prelude::*;

use crate::{
    services::{gateway::ApiGateway, sse::consume_app_sse},
    store::{
        AppEvent, ProtocolDetailsContext, RouteLookupContext, TracerouteResult,
        modal::ModalAction,
        ping::{PingAction, PingResult},
        traceroute::TracerouteAction,
    },
};

fn build_version_query(version: &str) -> String {
    match version {
        "4" => "&version=4".to_string(),
        "6" => "&version=6".to_string(),
        _ => String::new(),
    }
}

pub async fn perform_traceroute(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    version: String,
) -> Result<(), String> {
    state.dispatch(AppEvent::Traceroute(TracerouteAction::InitResult(
        node.clone(),
    )));

    if ApiGateway::send_ws_request(
        state,
        AppRequest::Traceroute {
            node: node.clone(),
            target: target.clone(),
            version: version.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/traceroute/{}?target={}{}",
        state.backend_url.trim_end_matches('/'),
        node,
        target,
        build_version_query(&version)
    );
    let state_for_stream = state.clone();

    if let Err(error) = consume_app_sse(url, move |response| {
        ApiGateway::dispatch_response(&state_for_stream, response);
    })
    .await
    {
        tracing::error!("Traceroute failed for {}: {}", node, error);
        state.dispatch(AppEvent::Traceroute(TracerouteAction::UpdateResult(
            node,
            TracerouteResult::Error(error.clone()),
        )));
        state.dispatch(AppEvent::Traceroute(TracerouteAction::EndOne));
        return Err(error);
    }

    Ok(())
}

pub async fn perform_ping(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    version: String,
) -> Result<(), String> {
    state.dispatch(AppEvent::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(format!(
            "{}@{}$ ping -c 5 {} {}",
            state.username, node, version, target
        )),
    }));

    if ApiGateway::send_ws_request(
        state,
        AppRequest::Ping {
            node: node.clone(),
            target: target.clone(),
            version: version.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/ping/{}?target={}{}",
        state.backend_url.trim_end_matches('/'),
        node,
        target,
        build_version_query(&version)
    );
    let state_for_stream = state.clone();

    if let Err(error) = consume_app_sse(url, move |response| {
        ApiGateway::dispatch_response(&state_for_stream, response);
    })
    .await
    {
        state.dispatch(AppEvent::Ping(PingAction::SetError(error.clone())));
        state.dispatch(AppEvent::PingModalUpdate {
            node,
            result: PingResult::Error(error.clone()),
        });
        state.dispatch(AppEvent::Ping(PingAction::End));
        return Err(error);
    }

    Ok(())
}

pub async fn perform_route_lookup(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    target: String,
    all: bool,
) -> Result<(), String> {
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

    if ApiGateway::send_ws_request(
        state,
        AppRequest::RouteLookup {
            node: node.clone(),
            target: target.clone(),
            all,
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/routes/{}?target={}&all={}",
        state.backend_url.trim_end_matches('/'),
        node,
        target,
        all
    );
    let state_for_stream = state.clone();

    if let Err(error) = consume_app_sse(url, move |response| {
        ApiGateway::dispatch_response(&state_for_stream, response);
    })
    .await
    {
        state.dispatch(AppEvent::Modal(ModalAction::UpdateContent(format!(
            "Failed to load route details: {}",
            error
        ))));
        return Err(error);
    }

    Ok(())
}

pub async fn get_protocols(state: &UseReducerHandle<crate::store::LgState>) -> Result<(), String> {
    if ApiGateway::send_ws_request(state, AppRequest::GetProtocols) {
        return Ok(());
    }

    let url = format!("{}/api/protocols", state.backend_url.trim_end_matches('/'));
    match ApiGateway::fetch_response(url).await {
        Ok(AppResponse::Error(error)) => Err(error),
        Ok(response) => {
            ApiGateway::dispatch_response(state, response);
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub async fn get_network_info(
    state: &UseReducerHandle<crate::store::LgState>,
) -> Result<(), String> {
    let url = format!("{}/api/info", state.backend_url.trim_end_matches('/'));
    match ApiGateway::fetch_response(url).await {
        Ok(AppResponse::Error(error)) => Err(error),
        Ok(response) => {
            ApiGateway::dispatch_response(state, response);
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub async fn get_protocol_details(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    proto: String,
) -> Result<(), String> {
    state.dispatch(AppEvent::SetProtocolDetailsContext(
        ProtocolDetailsContext {
            node: node.clone(),
            protocol: proto.clone(),
        },
    ));
    state.dispatch(AppEvent::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(format!(
            "{}@{}$ birdc show protocols all {}",
            state.username, node, proto
        )),
    }));

    if ApiGateway::send_ws_request(
        state,
        AppRequest::ProtocolDetails {
            node: node.clone(),
            protocol: proto.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/protocols/{}/{}",
        state.backend_url.trim_end_matches('/'),
        node,
        proto
    );
    let state_for_stream = state.clone();

    if let Err(error) = consume_app_sse(url, move |response| {
        ApiGateway::dispatch_response(&state_for_stream, response);
    })
    .await
    {
        state.dispatch(AppEvent::Modal(ModalAction::UpdateContent(format!(
            "Failed to load protocol details: {}",
            error
        ))));
        return Err(error);
    }

    Ok(())
}

pub async fn request_wireguard(
    state: &UseReducerHandle<crate::store::LgState>,
) -> Result<(), String> {
    if ApiGateway::send_ws_request(state, AppRequest::GetWireGuard) {
        return Ok(());
    }

    let url = format!("{}/api/wireguard", state.backend_url.trim_end_matches('/'));
    match ApiGateway::fetch_response(url).await {
        Ok(AppResponse::Error(error)) => {
            state.dispatch(AppEvent::SetError(format!(
                "WireGuard refresh failed: {}",
                error
            )));
            Err(error)
        }
        Ok(response) => {
            ApiGateway::dispatch_response(state, response);
            Ok(())
        }
        Err(error) => {
            state.dispatch(AppEvent::SetError(format!(
                "WireGuard refresh failed: {}",
                error
            )));
            Err(error)
        }
    }
}
