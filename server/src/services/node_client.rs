use chrono::Utc;
use common::{
    models::{NodeProtocol, NodeWireGuard},
    protocols::parse_protocols,
    utils::validate_target,
    wireguard::parse_wireguard_dump,
};
use ipnet::IpNet;
use tracing::warn;

use crate::{
    config::{Config, NodeConfig, PeeringInfo},
    services::request::{ByteStream, build_get, build_post, get_stream, post_stream},
};

#[derive(Clone)]
pub struct NodeClient {
    client: reqwest::Client,
}

impl NodeClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub fn resolve_node(&self, config: &Config, node_name: &str) -> Result<NodeConfig, String> {
        config
            .nodes
            .iter()
            .find(|node| node.name == node_name)
            .cloned()
            .ok_or_else(|| "Node not found".to_string())
    }

    fn ip_version_suffix(version: Option<&str>) -> &'static str {
        match version.unwrap_or_default() {
            "4" => "4",
            "6" => "6",
            _ => "",
        }
    }

    async fn stream_target_command(
        &self,
        node: &NodeConfig,
        target: &str,
        base_endpoint: &str,
        version: Option<&str>,
    ) -> Result<ByteStream, String> {
        validate_target(target)?;

        get_stream(
            &self.client,
            node,
            format!(
                "/{}{}?target={}",
                base_endpoint,
                Self::ip_version_suffix(version),
                target
            ),
        )
        .await
    }

    pub async fn traceroute_stream(
        &self,
        node: &NodeConfig,
        target: &str,
        version: Option<&str>,
    ) -> Result<ByteStream, String> {
        self.stream_target_command(node, target, "traceroute", version)
            .await
    }

    pub async fn ping_stream(
        &self,
        node: &NodeConfig,
        target: &str,
        version: Option<&str>,
    ) -> Result<ByteStream, String> {
        self.stream_target_command(node, target, "ping", version)
            .await
    }

    pub async fn route_lookup_stream(
        &self,
        node: &NodeConfig,
        target: &str,
        all: bool,
    ) -> Result<ByteStream, String> {
        let is_valid_target =
            target.parse::<std::net::IpAddr>().is_ok() || target.parse::<IpNet>().is_ok();

        if !is_valid_target {
            return Err("Invalid target format (must be IP or CIDR)".to_string());
        }

        let command = if all {
            format!("show route for {} all", target)
        } else {
            format!("show route for {}", target)
        };

        post_stream(&self.client, node, "/bird", &command).await
    }

    pub async fn peer_routes_stream(
        &self,
        node: &NodeConfig,
        peer: &str,
    ) -> Result<ByteStream, String> {
        let command = format!("show route protocol {}", peer);
        post_stream(&self.client, node, "/bird", &command).await
    }

    pub async fn protocol_details_stream(
        &self,
        node: &NodeConfig,
        protocol: &str,
    ) -> Result<ByteStream, String> {
        let command = format!("show protocols all {}", protocol);
        post_stream(&self.client, node, "/bird", &command).await
    }

    pub async fn fetch_wireguard_snapshot(&self, node: &NodeConfig) -> NodeWireGuard {
        let response = build_get(&self.client, node, "/wireguard").send().await;

        match response {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(dump_output) => NodeWireGuard {
                    name: node.name.clone(),
                    peers: parse_wireguard_dump(&dump_output),
                    last_updated: Utc::now(),
                    error: None,
                },
                Err(error) => {
                    warn!(node = %node.name, error = ?error, "Failed to read WireGuard response");
                    NodeWireGuard {
                        name: node.name.clone(),
                        peers: Vec::new(),
                        last_updated: Utc::now(),
                        error: Some("Failed to read response".to_string()),
                    }
                }
            },
            Ok(resp) => {
                warn!(node = %node.name, status = %resp.status(), "WireGuard endpoint returned error");
                NodeWireGuard {
                    name: node.name.clone(),
                    peers: Vec::new(),
                    last_updated: Utc::now(),
                    error: Some(format!("Node returned error: {}", resp.status())),
                }
            }
            Err(error) => {
                warn!(node = %node.name, error = ?error, "Failed to contact node for WireGuard info");
                NodeWireGuard {
                    name: node.name.clone(),
                    peers: Vec::new(),
                    last_updated: Utc::now(),
                    error: Some("Node is not reachable".to_string()),
                }
            }
        }
    }

    pub async fn fetch_protocol_snapshot(
        &self,
        node: &NodeConfig,
        current: Option<&NodeProtocol>,
    ) -> NodeProtocol {
        let response = build_post(&self.client, node, "/bird", "show protocols")
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(text) => NodeProtocol {
                    name: node.name.clone(),
                    protocols: parse_protocols(&text),
                    last_updated: Utc::now(),
                    error: None,
                },
                Err(error) => {
                    warn!(node = %node.name, error = ?error, "Failed to read BIRD response");
                    Self::cached_protocol_snapshot(
                        node,
                        current,
                        "Received invalid response from node.".to_string(),
                    )
                }
            },
            Ok(resp) => {
                warn!(node = %node.name, status = %resp.status(), "Node returned error status");
                Self::cached_protocol_snapshot(
                    node,
                    current,
                    format!("Node returned error: {}", resp.status()),
                )
            }
            Err(error) => {
                warn!(node = %node.name, error = ?error, "Failed to contact node");
                Self::cached_protocol_snapshot(
                    node,
                    current,
                    "Unable to reach node.".to_string(),
                )
            }
        }
    }

    pub async fn fetch_peering_info(&self, node: &NodeConfig) -> Option<PeeringInfo> {
        let response = build_get(&self.client, node, "/peering").send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Option<PeeringInfo>>().await {
                    Ok(peering) => peering,
                    Err(error) => {
                        warn!(node = %node.name, error = ?error, "Failed to parse peering info");
                        None
                    }
                }
            }
            Ok(resp) => {
                warn!(node = %node.name, status = %resp.status(), "Peering endpoint returned non-success status");
                None
            }
            Err(error) => {
                warn!(node = %node.name, error = ?error, "Failed to fetch peering info");
                None
            }
        }
    }

    fn cached_protocol_snapshot(
        node: &NodeConfig,
        current: Option<&NodeProtocol>,
        error: String,
    ) -> NodeProtocol {
        let has_cached = current.is_some_and(|c| !c.protocols.is_empty());
        let error_msg = if has_cached {
            format!("{error} Showing cached data.")
        } else {
            error
        };
        NodeProtocol {
            name: node.name.clone(),
            protocols: current
                .filter(|_| has_cached)
                .map(|value| value.protocols.clone())
                .unwrap_or_default(),
            last_updated: Utc::now(),
            error: Some(error_msg),
        }
    }
}
