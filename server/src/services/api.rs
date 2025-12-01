use std::pin::Pin;
use std::sync::Arc;

use common::{
    traceroute::{TracerouteHop, parse_traceroute_line},
    utils::validate_target,
};
use futures_util::{Stream, StreamExt, stream};
use ipnet::IpNet;
use std::net::IpAddr;
use tracing::warn;

use crate::config::Config;
use crate::services::request::{build_get, get_stream, post_stream};
use crate::state::{AppResponse, AppState};
use crate::utils::byte_stream_to_lines;

type BoxStream = Pin<Box<dyn Stream<Item = AppResponse> + Send>>;

fn stream_error(msg: String) -> BoxStream {
    Box::pin(stream::once(async move { AppResponse::Error(msg) }))
}

pub async fn perform_traceroute(
    state: AppState,
    config: Arc<Config>,
    node: String,
    target: String,
    version: Option<String>,
) -> BoxStream {
    let target_clone = target.clone();
    if let Err(msg) = validate_target(&target) {
        return stream_error(msg);
    }

    let node_config = match config.nodes.iter().find(|n| n.name == node).cloned() {
        Some(n) => n,
        None => return stream_error("Node not found".into()),
    };

    let endpoint = match version.as_deref().unwrap_or("") {
        "4" => "traceroute4",
        "6" => "traceroute6",
        _ => "traceroute",
    };
    let endpoint_with_query = format!("/{}?target={}", endpoint, target);
    let http_client = state.http_client.clone();

    match get_stream(&http_client, &node_config, &endpoint_with_query).await {
        Ok(byte_stream) => {
            let node_for_init = node.clone();
            let init = stream::once(async move {
                AppResponse::TracerouteInit {
                    node: node_for_init,
                }
            });

            let node_name = node.clone();
            let updates = byte_stream_to_lines(byte_stream).map(move |lines| {
                let hops: Vec<TracerouteHop> = lines
                    .into_iter()
                    .filter_map(|line| parse_traceroute_line(&line))
                    .collect();
                AppResponse::TracerouteUpdate {
                    node: node_name.clone(),
                    hops,
                }
            });

            Box::pin(init.chain(updates))
        }
        Err(err_msg) => {
            warn!(
                node = %node,
                target = %target_clone,
                error = %err_msg,
                "Failed to fetch traceroute information"
            );
            Box::pin(stream::once(async move {
                AppResponse::TracerouteError {
                    node,
                    error: err_msg,
                }
            }))
        }
    }
}

pub async fn perform_route_lookup(
    state: AppState,
    config: Arc<Config>,
    node: String,
    target: String,
    all: bool,
) -> BoxStream {
    let target_clone = target.clone();
    let is_valid_target = target.parse::<IpAddr>().is_ok() || target.parse::<IpNet>().is_ok();

    if !is_valid_target {
        return stream_error("Invalid target format (must be IP or CIDR)".into());
    }

    let node_config = match config.nodes.iter().find(|n| n.name == node).cloned() {
        Some(n) => n,
        None => return stream_error("Node not found".into()),
    };

    let command = if all {
        format!("show route for {} all", target)
    } else {
        format!("show route for {}", target)
    };

    let http_client = state.http_client.clone();

    match post_stream(&http_client, &node_config, "/bird", &command).await {
        Ok(byte_stream) => {
            let node_for_init = node.clone();
            let init = stream::once(async move {
                AppResponse::RouteLookupInit {
                    node: node_for_init,
                }
            });

            let node_name = node.clone();
            let updates = byte_stream_to_lines(byte_stream).map(move |lines| {
                AppResponse::RouteLookupUpdate {
                    node: node_name.clone(),
                    lines,
                }
            });
            Box::pin(init.chain(updates))
        }
        Err(err_msg) => {
            warn!(
                node = %node,
                target = %target_clone,
                error = %err_msg,
                "Failed to fetch route information"
            );
            stream_error(err_msg)
        }
    }
}

pub async fn get_protocol_details(
    state: AppState,
    config: Arc<Config>,
    node: String,
    protocol: String,
) -> BoxStream {
    let node_config = match config.nodes.iter().find(|n| n.name == node).cloned() {
        Some(n) => n,
        None => return stream_error("Node not found".into()),
    };

    let command = format!("show protocols all {}", protocol);
    let http_client = state.http_client.clone();

    match post_stream(&http_client, &node_config, "/bird", &command).await {
        Ok(byte_stream) => {
            let node_for_init = node.clone();
            let protocol_for_init = protocol.clone();
            let init = stream::once(async move {
                AppResponse::ProtocolDetailsInit {
                    node: node_for_init,
                    protocol: protocol_for_init,
                }
            });

            let node_name = node.clone();
            let protocol_name = protocol.clone();
            let updates = byte_stream_to_lines(byte_stream).map(move |lines| {
                AppResponse::ProtocolDetailsUpdate {
                    node: node_name.clone(),
                    protocol: protocol_name.clone(),
                    lines,
                }
            });

            Box::pin(init.chain(updates))
        }
        Err(err_msg) => {
            warn!(
                node = %node,
                protocol = %protocol,
                error = %err_msg,
                "Failed to fetch protocol details"
            );
            stream_error(err_msg)
        }
    }
}

pub async fn get_wireguard(state: AppState, config: Arc<Config>) -> BoxStream {
    use chrono::Utc;
    use common::models::NodeWireGuard;
    use common::wireguard::parse_wireguard_dump;

    let http_client = state.http_client.clone();
    let mut wireguard_data = Vec::new();

    for node in &config.nodes {
        let req = build_get(&http_client, node, "/wireguard");
        match req.send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(dump_output) => {
                    let peers = parse_wireguard_dump(&dump_output);
                    wireguard_data.push(NodeWireGuard {
                        name: node.name.clone(),
                        peers,
                        last_updated: Utc::now(),
                        error: None,
                    });
                }
                Err(e) => {
                    warn!(node = %node.name, error = ?e, "Failed to read WireGuard response");
                    wireguard_data.push(NodeWireGuard {
                        name: node.name.clone(),
                        peers: Vec::new(),
                        last_updated: Utc::now(),
                        error: Some("Failed to read response".to_string()),
                    });
                }
            },
            Ok(resp) => {
                warn!(node = %node.name, status = %resp.status(), "WireGuard endpoint returned error");
                wireguard_data.push(NodeWireGuard {
                    name: node.name.clone(),
                    peers: Vec::new(),
                    last_updated: Utc::now(),
                    error: Some(format!("Node returned error: {}", resp.status())),
                });
            }
            Err(e) => {
                warn!(node = %node.name, error = ?e, "Failed to contact node for WireGuard info");
                wireguard_data.push(NodeWireGuard {
                    name: node.name.clone(),
                    peers: Vec::new(),
                    last_updated: Utc::now(),
                    error: Some("Node is not reachable".to_string()),
                });
            }
        }
    }

    Box::pin(stream::once(async move {
        AppResponse::WireGuard {
            data: wireguard_data,
        }
    }))
}
