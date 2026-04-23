use std::sync::atomic::{AtomicU64, Ordering};

use common::api::{AppRequest, AppResponse};
use futures::future::join_all;
use yew::prelude::*;

use crate::{
    services::{gateway::ApiGateway, sse::consume_app_sse},
    store::{AppEvent, command_output::CommandOutputKind},
};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn current_timestamp_ms() -> u64 {
    web_sys::js_sys::Date::now() as u64
}

fn build_request_id(prefix: &str) -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = current_timestamp_ms();

    format!("{prefix}-{timestamp}-{counter}")
}

fn build_version_query(version: &str) -> String {
    match version {
        "4" => "&version=4".to_string(),
        "6" => "&version=6".to_string(),
        _ => String::new(),
    }
}

fn build_version_flag(version: &str) -> &'static str {
    match version {
        "4" => " -4",
        "6" => " -6",
        _ => "",
    }
}

fn dispatch_streamed_response(
    state: &UseReducerHandle<crate::store::LgState>,
    url: String,
) -> impl std::future::Future<Output = Result<(), String>> {
    let state_for_stream = state.clone();

    async move {
        consume_app_sse(url, move |response| {
            ApiGateway::dispatch_response(&state_for_stream, response);
        })
        .await
    }
}

pub async fn perform_traceroute(
    state: &UseReducerHandle<crate::store::LgState>,
    target_nodes: Vec<String>,
    target: String,
    version: String,
) -> Result<(), String> {
    if target_nodes.is_empty() {
        return Err("No nodes available".to_string());
    }

    let request_id = build_request_id("traceroute");
    state.dispatch(AppEvent::StartTraceroute {
        request_id: request_id.clone(),
        target: target.clone(),
        version: version.clone(),
        pending: target_nodes.len(),
    });

    let futures = target_nodes.into_iter().map(|node| {
        send_traceroute_request(
            state.clone(),
            request_id.clone(),
            node,
            target.clone(),
            version.clone(),
        )
    });

    let results = join_all(futures).await;
    if let Some(error) = results.into_iter().find_map(Result::err) {
        return Err(error);
    }

    Ok(())
}

async fn send_traceroute_request(
    state: UseReducerHandle<crate::store::LgState>,
    request_id: String,
    node: String,
    target: String,
    version: String,
) -> Result<(), String> {
    if ApiGateway::send_ws_request(
        &state,
        AppRequest::Traceroute {
            request_id: request_id.clone(),
            node: node.clone(),
            target: target.clone(),
            version: version.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/traceroute/{}?request_id={}&target={}{}",
        state.backend_url.trim_end_matches('/'),
        node,
        request_id,
        target,
        build_version_query(&version)
    );

    if let Err(error) = dispatch_streamed_response(&state, url).await {
        tracing::error!("Traceroute failed for {}: {}", node, error);
        ApiGateway::dispatch_response(
            &state,
            AppResponse::TracerouteError {
                request_id,
                node,
                error: error.clone(),
            },
        );
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
    let request_id = build_request_id("ping");
    state.dispatch(AppEvent::StartPing {
        request_id: request_id.clone(),
        node: node.clone(),
        target: target.clone(),
        version: version.clone(),
        command: format!(
            "{}@{}$ ping -c 5{} {}",
            state.username,
            node,
            build_version_flag(&version),
            target
        ),
    });

    if ApiGateway::send_ws_request(
        state,
        AppRequest::Ping {
            request_id: request_id.clone(),
            node: node.clone(),
            target: target.clone(),
            version: version.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/ping/{}?request_id={}&target={}{}",
        state.backend_url.trim_end_matches('/'),
        node,
        request_id,
        target,
        build_version_query(&version)
    );

    if let Err(error) = dispatch_streamed_response(state, url).await {
        ApiGateway::dispatch_response(
            state,
            AppResponse::PingError {
                request_id,
                node,
                error: error.clone(),
            },
        );
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
    let request_id = build_request_id("route-lookup");
    let command = if all {
        format!(
            "{}@{}$ birdc show route {} all",
            state.username, node, target
        )
    } else {
        format!("{}@{}$ birdc show route {}", state.username, node, target)
    };

    state.dispatch(AppEvent::StartCommandOutput {
        request_id: request_id.clone(),
        kind: CommandOutputKind::RouteLookup,
        command,
    });

    if ApiGateway::send_ws_request(
        state,
        AppRequest::RouteLookup {
            request_id: request_id.clone(),
            node: node.clone(),
            target: target.clone(),
            all,
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/routes/{}?request_id={}&target={}&all={}",
        state.backend_url.trim_end_matches('/'),
        node,
        request_id,
        target,
        all
    );

    if let Err(error) = dispatch_streamed_response(state, url).await {
        ApiGateway::dispatch_response(
            state,
            AppResponse::RouteLookupError {
                request_id,
                node,
                error: error.clone(),
            },
        );
        return Err(error);
    }

    Ok(())
}

pub async fn perform_peer_routes(
    state: &UseReducerHandle<crate::store::LgState>,
    node: String,
    peer: String,
) -> Result<(), String> {
    let request_id = build_request_id("peer-routes");
    let command = format!(
        "{}@{}$ birdc show route protocol {}",
        state.username, node, peer
    );

    state.dispatch(AppEvent::StartCommandOutput {
        request_id: request_id.clone(),
        kind: CommandOutputKind::RouteLookup,
        command,
    });

    if ApiGateway::send_ws_request(
        state,
        AppRequest::PeerRoutes {
            request_id: request_id.clone(),
            node: node.clone(),
            peer: peer.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/routes/{}/peer/{}?request_id={}",
        state.backend_url.trim_end_matches('/'),
        node,
        peer,
        request_id
    );

    if let Err(error) = dispatch_streamed_response(state, url).await {
        ApiGateway::dispatch_response(
            state,
            AppResponse::RouteLookupError {
                request_id,
                node,
                error: error.clone(),
            },
        );
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
    let request_id = build_request_id("protocol-details");
    state.dispatch(AppEvent::StartCommandOutput {
        request_id: request_id.clone(),
        kind: CommandOutputKind::ProtocolDetails,
        command: format!(
            "{}@{}$ birdc show protocols all {}",
            state.username, node, proto
        ),
    });

    if ApiGateway::send_ws_request(
        state,
        AppRequest::ProtocolDetails {
            request_id: request_id.clone(),
            node: node.clone(),
            protocol: proto.clone(),
        },
    ) {
        return Ok(());
    }

    let url = format!(
        "{}/api/protocols/{}/{}?request_id={}",
        state.backend_url.trim_end_matches('/'),
        node,
        proto,
        request_id
    );

    if let Err(error) = dispatch_streamed_response(state, url).await {
        ApiGateway::dispatch_response(
            state,
            AppResponse::ProtocolDetailsError {
                request_id,
                node,
                protocol: proto,
                error: error.clone(),
            },
        );
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
