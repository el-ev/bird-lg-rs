use std::{sync::Arc, time::Duration};

use chrono::Utc;
use common::models::NodeStatusDiff;
use tokio::time::sleep;
use tracing::warn;

use crate::{
    config::{Config, NodeConfig, PeeringInfo},
    diff::calculate_diff,
    services::request::{build_get, build_post},
    state::{AppResponse, AppState, NodeProtocol},
    utils::parse_protocols,
};

pub fn spawn(state: AppState, config: Arc<Config>) {
    tokio::spawn(run(state, config));
}

async fn run(state: AppState, config: Arc<Config>) {
    let client = create_client();

    let mut poll_counter = 0u32;
    const PEERING_POLL_INTERVAL: u32 = 180;

    loop {
        if check_idle_timeout(&state, &config).await {
            continue;
        }

        let mut new_statuses = Vec::new();
        let current_nodes = { state.nodes.read().unwrap().clone() };

        let should_fetch_peering = poll_counter.is_multiple_of(PEERING_POLL_INTERVAL);

        for node in &config.nodes {
            let status =
                process_node(&client, node, &state, &current_nodes, should_fetch_peering).await;
            new_statuses.push(status);
        }

        broadcast_updates(&state, new_statuses, &current_nodes);

        poll_counter = poll_counter.wrapping_add(1);
        sleep(Duration::from_secs(10)).await;
    }
}

fn create_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|e| {
            warn!(error = ?e, "Failed to build reqwest client with timeout, falling back");
            reqwest::Client::new()
        })
}

async fn check_idle_timeout(state: &AppState, config: &Config) -> bool {
    if state
        .active_connections
        .load(std::sync::atomic::Ordering::Relaxed)
        > 0
    {
        if !state
            .is_polling_active
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            state
                .is_polling_active
                .store(true, std::sync::atomic::Ordering::Relaxed);
            tracing::info!("Resuming polling due to active WebSocket connection");
        }
        return false;
    }

    if let Some(idle_timeout_secs) = config.poll_idle_timeout {
        let should_pause = {
            let last_req = state.last_request_time.read().unwrap();
            last_req
                .map(|t| t.elapsed().as_secs() > idle_timeout_secs)
                .unwrap_or(false)
        };

        if should_pause
            && state
                .is_polling_active
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            state
                .is_polling_active
                .store(false, std::sync::atomic::Ordering::Relaxed);
            tracing::info!(idle_timeout_secs, "Pausing polling due to inactivity");
        }

        if !state
            .is_polling_active
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            sleep(Duration::from_secs(5)).await;
            return true;
        } else if should_pause {
            tracing::info!("Resuming polling after request activity");
        }
    }
    false
}

async fn process_node(
    client: &reqwest::Client,
    node: &NodeConfig,
    state: &AppState,
    current_nodes: &[NodeProtocol],
    should_fetch_peering: bool,
) -> NodeProtocol {
    let command = "show protocols";
    let req = build_post(client, node, "/bird", command);
    let resp = req.send().await;

    if (should_fetch_peering || !state.peering.read().unwrap().contains_key(&node.name))
        && let Some(info) = fetch_peering_info(client, node).await
    {
        state
            .peering
            .write()
            .unwrap()
            .insert(node.name.clone(), info);
    }

    match resp {
        Ok(r) => {
            if !r.status().is_success() {
                warn!(node = %node.name, status = %r.status(), "Node returned error status");
                let existing = current_nodes.iter().find(|n| n.name == node.name);
                return NodeProtocol {
                    name: node.name.clone(),
                    protocols: existing.map(|n| n.protocols.clone()).unwrap_or_default(),
                    last_updated: Utc::now(),
                    error: Some(format!("Node returned error: {}", r.status())),
                };
            }

            match r.text().await {
                Ok(text) => {
                    let protocols = parse_protocols(&text);

                    NodeProtocol {
                        name: node.name.clone(),
                        protocols,
                        last_updated: Utc::now(),
                        error: None,
                    }
                }
                Err(e) => {
                    warn!(node = %node.name, error = ?e, "Failed to read BIRD response");
                    let existing = current_nodes.iter().find(|n| n.name == node.name);
                    NodeProtocol {
                        name: node.name.clone(),
                        protocols: existing.map(|n| n.protocols.clone()).unwrap_or_default(),
                        last_updated: Utc::now(),
                        error: Some(
                            "Received invalid response from node. Showing cached data.".into(),
                        ),
                    }
                }
            }
        }
        Err(e) => {
            warn!(node = %node.name, error = ?e, "Failed to contact node");
            let existing = current_nodes.iter().find(|n| n.name == node.name);
            NodeProtocol {
                name: node.name.clone(),
                protocols: existing.map(|n| n.protocols.clone()).unwrap_or_default(),
                last_updated: Utc::now(),
                error: Some("Unable to reach node. Showing cached data.".into()),
            }
        }
    }
}

fn broadcast_updates(
    state: &AppState,
    new_statuses: Vec<NodeProtocol>,
    current_nodes: &[NodeProtocol],
) {
    let changed = if new_statuses.len() != current_nodes.len() {
        true
    } else {
        new_statuses
            .iter()
            .zip(current_nodes.iter())
            .any(|(new, old)| {
                new.name != old.name || new.protocols != old.protocols || new.error != old.error
            })
            || new_statuses.iter().any(|n| n.error.is_some())
    };

    {
        let mut w = state.nodes.write().unwrap();
        *w = new_statuses.clone();
    }

    let resp = if changed {
        let diffs: Vec<NodeStatusDiff> = new_statuses
            .iter()
            .zip(current_nodes.iter())
            .map(|(new, old)| NodeStatusDiff {
                n: new.name.clone(),
                d: calculate_diff(&old.protocols, &new.protocols),
                u: new.last_updated,
                e: new.error.clone(),
            })
            .collect();
        AppResponse::ProtocolsDiff { data: diffs }
    } else {
        AppResponse::NoChange {
            last_updated: Utc::now(),
        }
    };

    let _ = state.tx.send(resp);
}

async fn fetch_peering_info(client: &reqwest::Client, node: &NodeConfig) -> Option<PeeringInfo> {
    let req = build_get(client, node, "/peering");

    match req.send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Option<PeeringInfo>>().await {
            Ok(peering) => peering,
            Err(e) => {
                warn!(node = %node.name, error = ?e, "Failed to parse peering info");
                None
            }
        },
        Ok(resp) => {
            warn!(node = %node.name, status = %resp.status(), "Peering endpoint returned non-success status");
            None
        }
        Err(e) => {
            warn!(node = %node.name, error = ?e, "Failed to fetch peering info");
            None
        }
    }
}
