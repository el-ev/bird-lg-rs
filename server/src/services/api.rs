use std::pin::Pin;
use std::sync::Arc;

use common::{utils::parse_traceroute_line, validate_target};
use futures_util::{Stream, StreamExt, stream};
use ipnet::IpNet;
use std::net::IpAddr;
use tracing::warn;

use crate::config::Config;
use crate::services::request::{get_stream, post_stream};
use crate::state::{AppResponse, AppState};

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
            let updates = byte_stream_to_lines(byte_stream).flat_map(move |lines| {
                let node_name = node_name.clone();
                stream::iter(lines.into_iter().filter_map(move |line| {
                    parse_traceroute_line(&line).map(|hop| AppResponse::TracerouteUpdate {
                        node: node_name.clone(),
                        hop,
                    })
                }))
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
            let updates =
                byte_stream_to_lines(byte_stream).flat_map(move |lines| {
                    let node_name = node_name.clone();
                    stream::iter(lines.into_iter().map(move |line| {
                        AppResponse::RouteLookupUpdate {
                            node: node_name.clone(),
                            line,
                        }
                    }))
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
            let updates = byte_stream_to_lines(byte_stream).flat_map(move |lines| {
                let node_name = node_name.clone();
                let protocol_name = protocol_name.clone();
                stream::iter(lines.into_iter().map(move |line| {
                    AppResponse::ProtocolDetailsUpdate {
                        node: node_name.clone(),
                        protocol: protocol_name.clone(),
                        line,
                    }
                }))
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

fn byte_stream_to_lines<S, E>(stream: S) -> impl Stream<Item = Vec<String>>
where
    S: Stream<Item = Result<axum::body::Bytes, E>> + Unpin,
    E: std::fmt::Debug,
{
    stream::unfold((stream, Vec::new()), |(mut stream, mut buf)| async move {
        loop {
            let extract_lines = |buffer: &mut Vec<u8>| -> Vec<String> {
                let mut lines = Vec::new();
                while let Some(i) = buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes: Vec<u8> = buffer.drain(..=i).collect();
                    let mut line = String::from_utf8_lossy(&line_bytes).to_string();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    lines.push(line);
                }
                lines
            };

            match stream.next().await {
                Some(Ok(bytes)) => {
                    buf.extend_from_slice(&bytes);

                    if buf.contains(&b'\n') {
                        let lines = extract_lines(&mut buf);
                        if !lines.is_empty() {
                            return Some((lines, (stream, buf)));
                        }
                    }
                }
                Some(Err(_)) => {
                    let lines = extract_lines(&mut buf);
                    if !lines.is_empty() {
                        return Some((lines, (stream, buf)));
                    }
                    return None;
                }
                None => {
                    let mut lines = extract_lines(&mut buf);
                    if !buf.is_empty() {
                        let line = String::from_utf8_lossy(&buf).to_string();
                        lines.push(line);
                        buf.clear();
                    }

                    if !lines.is_empty() {
                        return Some((lines, (stream, buf)));
                    }
                    return None;
                }
            }
        }
    })
}
