use std::{pin::Pin, sync::Arc};

use common::{
    api::AppResponse,
    traceroute::{TracerouteHop, parse_traceroute_line},
};
use futures_util::{Stream, StreamExt, stream};
use tracing::warn;

use crate::{
    config::Config, services::node_client::NodeClient, state::AppState, utils::byte_stream_to_lines,
};

type BoxStream = Pin<Box<dyn Stream<Item = AppResponse> + Send>>;

fn boxed_once(response: AppResponse) -> BoxStream {
    Box::pin(stream::once(async move { response }))
}

pub async fn perform_traceroute(
    state: AppState,
    config: Arc<Config>,
    node: String,
    target: String,
    version: Option<String>,
) -> BoxStream {
    let client = NodeClient::new(state.http_client.clone());
    let node_config = match client.resolve_node(&config, &node) {
        Ok(node_config) => node_config,
        Err(error) => {
            return boxed_once(AppResponse::TracerouteError { node, error });
        }
    };

    match client
        .traceroute_stream(&node_config, &target, version.as_deref())
        .await
    {
        Ok(byte_stream) => {
            let init = stream::once({
                let node = node.clone();
                async move { AppResponse::TracerouteInit { node } }
            });

            let updates = byte_stream_to_lines(byte_stream).map({
                let node = node.clone();
                move |lines| {
                    let hops: Vec<TracerouteHop> = lines
                        .into_iter()
                        .filter_map(|line| parse_traceroute_line(&line))
                        .collect();

                    AppResponse::TracerouteUpdate {
                        node: node.clone(),
                        hops,
                    }
                }
            });

            let done = stream::once(async move { AppResponse::TracerouteDone { node } });
            Box::pin(init.chain(updates).chain(done))
        }
        Err(error) => {
            warn!(
                node = %node,
                target = %target,
                error = %error,
                "Failed to fetch traceroute information"
            );
            boxed_once(AppResponse::TracerouteError { node, error })
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
    let client = NodeClient::new(state.http_client.clone());
    let node_config = match client.resolve_node(&config, &node) {
        Ok(node_config) => node_config,
        Err(error) => {
            return boxed_once(AppResponse::RouteLookupError { node, error });
        }
    };

    match client.route_lookup_stream(&node_config, &target, all).await {
        Ok(byte_stream) => {
            let init = stream::once({
                let node = node.clone();
                async move { AppResponse::RouteLookupInit { node } }
            });

            let updates = byte_stream_to_lines(byte_stream).map({
                let node = node.clone();
                move |lines| AppResponse::RouteLookupUpdate {
                    node: node.clone(),
                    lines,
                }
            });

            let done = stream::once(async move { AppResponse::RouteLookupDone { node } });
            Box::pin(init.chain(updates).chain(done))
        }
        Err(error) => {
            warn!(
                node = %node,
                target = %target,
                error = %error,
                "Failed to fetch route information"
            );
            boxed_once(AppResponse::RouteLookupError { node, error })
        }
    }
}

pub async fn get_protocol_details(
    state: AppState,
    config: Arc<Config>,
    node: String,
    protocol: String,
) -> BoxStream {
    let client = NodeClient::new(state.http_client.clone());
    let node_config = match client.resolve_node(&config, &node) {
        Ok(node_config) => node_config,
        Err(error) => {
            return boxed_once(AppResponse::ProtocolDetailsError {
                node,
                protocol,
                error,
            });
        }
    };

    match client
        .protocol_details_stream(&node_config, &protocol)
        .await
    {
        Ok(byte_stream) => {
            let init = stream::once({
                let node = node.clone();
                let protocol = protocol.clone();
                async move { AppResponse::ProtocolDetailsInit { node, protocol } }
            });

            let updates = byte_stream_to_lines(byte_stream).map({
                let node = node.clone();
                let protocol = protocol.clone();
                move |lines| AppResponse::ProtocolDetailsUpdate {
                    node: node.clone(),
                    protocol: protocol.clone(),
                    lines,
                }
            });

            let done =
                stream::once(async move { AppResponse::ProtocolDetailsDone { node, protocol } });
            Box::pin(init.chain(updates).chain(done))
        }
        Err(error) => {
            warn!(
                node = %node,
                protocol = %protocol,
                error = %error,
                "Failed to fetch protocol details"
            );
            boxed_once(AppResponse::ProtocolDetailsError {
                node,
                protocol,
                error,
            })
        }
    }
}

pub async fn get_wireguard(state: AppState, config: Arc<Config>) -> BoxStream {
    let client = NodeClient::new(state.http_client.clone());
    let mut wireguard_data = Vec::new();

    for node in &config.nodes {
        wireguard_data.push(client.fetch_wireguard_snapshot(node).await);
    }

    boxed_once(AppResponse::WireGuard {
        data: wireguard_data,
    })
}

pub async fn perform_ping(
    state: AppState,
    config: Arc<Config>,
    node: String,
    target: String,
    version: Option<String>,
) -> BoxStream {
    let client = NodeClient::new(state.http_client.clone());
    let node_config = match client.resolve_node(&config, &node) {
        Ok(node_config) => node_config,
        Err(error) => {
            return boxed_once(AppResponse::PingError { node, error });
        }
    };

    match client
        .ping_stream(&node_config, &target, version.as_deref())
        .await
    {
        Ok(byte_stream) => {
            let init = stream::once({
                let node = node.clone();
                async move { AppResponse::PingInit { node } }
            });

            let updates = byte_stream_to_lines(byte_stream).map({
                let node = node.clone();
                move |lines| AppResponse::PingUpdate {
                    node: node.clone(),
                    lines,
                }
            });

            let done = stream::once(async move { AppResponse::PingDone { node } });
            Box::pin(init.chain(updates).chain(done))
        }
        Err(error) => {
            warn!(
                node = %node,
                target = %target,
                error = %error,
                "Failed to fetch ping information"
            );
            boxed_once(AppResponse::PingError { node, error })
        }
    }
}
