use std::net::IpAddr;
use std::sync::Arc;

use common::validate_target;
use ipnet::IpNet;
use tracing::warn;

use crate::config::Config;
use crate::services::request::{get_text, post_text};
use crate::state::{AppResponse, AppState};

pub async fn perform_traceroute(
    state: &AppState,
    config: &Arc<Config>,
    node: &str,
    target: &str,
    version: Option<&str>,
) -> AppResponse {
    let target = target.trim();
    if let Err(msg) = validate_target(target) {
        return AppResponse::Error(msg);
    }

    if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
        let endpoint = match version.unwrap_or("4") {
            "4" => "traceroute4",
            "6" => "traceroute6",
            _ => "traceroute",
        };
        let endpoint_with_query = format!("/{}?target={}", endpoint, target);

        match get_text(&state.http_client, node_config, &endpoint_with_query).await {
            Ok(text) => AppResponse::TracerouteResult {
                node: node.to_string(),
                result: text,
            },
            Err(err_msg) => {
                warn!(
                    node = %node,
                    target = %target,
                    error = %err_msg,
                    "Failed to fetch traceroute information"
                );
                AppResponse::Error(err_msg)
            }
        }
    } else {
        AppResponse::Error("Node not found".into())
    }
}

pub async fn perform_route_lookup(
    state: &AppState,
    config: &Arc<Config>,
    node: &str,
    target: &str,
    all: bool,
) -> AppResponse {
    let target = target.trim();
    let is_valid_target = target.parse::<IpAddr>().is_ok() || target.parse::<IpNet>().is_ok();

    if !is_valid_target {
        return AppResponse::Error("Invalid target format (must be IP or CIDR)".into());
    }

    if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
        let command = if all {
            format!("show route for {} all", target)
        } else {
            format!("show route for {}", target)
        };

        match post_text(&state.http_client, node_config, "/bird", &command).await {
            Ok(text) => AppResponse::RouteLookupResult {
                node: node.to_string(),
                result: text,
            },
            Err(err_msg) => {
                warn!(
                    node = %node,
                    target = %target,
                    error = %err_msg,
                    "Failed to fetch route information"
                );
                AppResponse::Error(err_msg)
            }
        }
    } else {
        AppResponse::Error("Node not found".into())
    }
}

pub async fn get_protocol_details(
    state: &AppState,
    config: &Arc<Config>,
    node: &str,
    protocol: &str,
) -> AppResponse {
    if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
        let command = format!("show protocols all {}", protocol);

        match post_text(&state.http_client, node_config, "/bird", &command).await {
            Ok(text) => AppResponse::ProtocolDetailsResult {
                node: node.to_string(),
                protocol: protocol.to_string(),
                details: text,
            },
            Err(err_msg) => {
                warn!(
                    node = %node,
                    protocol = %protocol,
                    error = %err_msg,
                    "Failed to fetch protocol details"
                );
                AppResponse::Error(err_msg)
            }
        }
    } else {
        AppResponse::Error("Node not found".into())
    }
}
