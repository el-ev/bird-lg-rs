use std::{pin::Pin, sync::Arc};

use common::{
    api::AppResponse,
    traceroute::{TracerouteHop, parse_traceroute_line},
};
use futures_util::{Stream, StreamExt, stream};
use tracing::warn;

use crate::{
    config::{Config, NodeConfig},
    services::{node_client::NodeClient, request::ByteStream},
    state::AppState,
    utils::byte_stream_to_lines,
};

type BoxStream = Pin<Box<dyn Stream<Item = AppResponse> + Send>>;

fn boxed_once(response: AppResponse) -> BoxStream {
    Box::pin(stream::once(async move { response }))
}

fn resolve_node_or_error<F>(
    client: &NodeClient,
    config: &Config,
    node: &str,
    error_response: F,
) -> Result<NodeConfig, BoxStream>
where
    F: FnOnce(String) -> AppResponse,
{
    client
        .resolve_node(config, node)
        .map_err(|error| boxed_once(error_response(error)))
}

fn stream_with_terminal_events<F>(
    byte_stream: ByteStream,
    init: AppResponse,
    update: F,
    done: AppResponse,
) -> BoxStream
where
    F: Fn(Vec<String>) -> AppResponse + Send + 'static,
{
    let init = stream::once(async move { init });
    let updates = byte_stream_to_lines(byte_stream).map(update);
    let done = stream::once(async move { done });

    Box::pin(init.chain(updates).chain(done))
}

pub async fn perform_traceroute(
    state: AppState,
    config: Arc<Config>,
    node: String,
    target: String,
    version: Option<String>,
) -> BoxStream {
    let client = NodeClient::new(state.http_client.clone());
    let node_config = match resolve_node_or_error(&client, &config, &node, |error| {
        AppResponse::TracerouteError {
            node: node.clone(),
            error,
        }
    }) {
        Ok(node_config) => node_config,
        Err(response) => {
            return response;
        }
    };

    match client
        .traceroute_stream(&node_config, &target, version.as_deref())
        .await
    {
        Ok(byte_stream) => stream_with_terminal_events(
            byte_stream,
            AppResponse::TracerouteInit { node: node.clone() },
            {
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
            },
            AppResponse::TracerouteDone { node },
        ),
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
    let node_config = match resolve_node_or_error(&client, &config, &node, |error| {
        AppResponse::RouteLookupError {
            node: node.clone(),
            error,
        }
    }) {
        Ok(node_config) => node_config,
        Err(response) => {
            return response;
        }
    };

    match client.route_lookup_stream(&node_config, &target, all).await {
        Ok(byte_stream) => stream_with_terminal_events(
            byte_stream,
            AppResponse::RouteLookupInit { node: node.clone() },
            {
                let node = node.clone();
                move |lines| AppResponse::RouteLookupUpdate {
                    node: node.clone(),
                    lines,
                }
            },
            AppResponse::RouteLookupDone { node },
        ),
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
    let node_config = match resolve_node_or_error(&client, &config, &node, |error| {
        AppResponse::ProtocolDetailsError {
            node: node.clone(),
            protocol: protocol.clone(),
            error,
        }
    }) {
        Ok(node_config) => node_config,
        Err(response) => {
            return response;
        }
    };

    match client
        .protocol_details_stream(&node_config, &protocol)
        .await
    {
        Ok(byte_stream) => {
            let init_node = node.clone();
            let init_protocol = protocol.clone();

            stream_with_terminal_events(
                byte_stream,
                AppResponse::ProtocolDetailsInit {
                    node: init_node,
                    protocol: init_protocol,
                },
                {
                    let node = node.clone();
                    let protocol = protocol.clone();
                    move |lines| AppResponse::ProtocolDetailsUpdate {
                        node: node.clone(),
                        protocol: protocol.clone(),
                        lines,
                    }
                },
                AppResponse::ProtocolDetailsDone { node, protocol },
            )
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
    let node_config =
        match resolve_node_or_error(&client, &config, &node, |error| AppResponse::PingError {
            node: node.clone(),
            error,
        }) {
            Ok(node_config) => node_config,
            Err(response) => {
                return response;
            }
        };

    match client
        .ping_stream(&node_config, &target, version.as_deref())
        .await
    {
        Ok(byte_stream) => stream_with_terminal_events(
            byte_stream,
            AppResponse::PingInit { node: node.clone() },
            {
                let node = node.clone();
                move |lines| AppResponse::PingUpdate {
                    node: node.clone(),
                    lines,
                }
            },
            AppResponse::PingDone { node },
        ),
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

#[cfg(test)]
mod tests {
    use axum::body::Bytes;
    use common::api::AppResponse;
    use futures_util::{StreamExt, stream};

    use super::{ByteStream, stream_with_terminal_events};

    #[tokio::test]
    async fn stream_with_terminal_events_preserves_init_update_done_order() {
        let byte_stream: ByteStream = Box::pin(stream::iter([
            Ok::<_, std::io::Error>(Bytes::from("line one\nline two\n")),
            Ok(Bytes::from("line three")),
        ]));

        let responses = stream_with_terminal_events(
            byte_stream,
            AppResponse::PingInit {
                node: "edge-a".to_string(),
            },
            |lines| AppResponse::PingUpdate {
                node: "edge-a".to_string(),
                lines,
            },
            AppResponse::PingDone {
                node: "edge-a".to_string(),
            },
        )
        .collect::<Vec<_>>()
        .await;

        assert_eq!(responses.len(), 4);
        assert!(matches!(
            &responses[0],
            AppResponse::PingInit { node } if node == "edge-a"
        ));
        assert!(matches!(
            &responses[1],
            AppResponse::PingUpdate { node, lines }
                if node == "edge-a" && lines == &vec!["line one".to_string(), "line two".to_string()]
        ));
        assert!(matches!(
            &responses[2],
            AppResponse::PingUpdate { node, lines }
                if node == "edge-a" && lines == &vec!["line three".to_string()]
        ));
        assert!(matches!(
            &responses[3],
            AppResponse::PingDone { node } if node == "edge-a"
        ));
    }
}
