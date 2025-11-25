use std::{sync::Arc, time::Duration};

use chrono::Utc;
use tokio::time::sleep;
use tracing::warn;

use crate::{
    config::{Config, PeeringInfo},
    parser,
    state::{AppState, NodeStatus},
};

pub fn spawn(state: AppState, config: Arc<Config>) {
    tokio::spawn(run(state, config));
}

async fn run(state: AppState, config: Arc<Config>) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|e| {
            warn!(error = ?e, "Failed to build reqwest client with timeout, falling back");
            reqwest::Client::new()
        });

    let mut poll_counter = 0u32;
    const PEERING_POLL_INTERVAL: u32 = 180;
    loop {
        let mut new_statuses = Vec::new();
        let current_nodes = { state.nodes.read().unwrap().clone() };

        let should_fetch_peering = poll_counter.is_multiple_of(PEERING_POLL_INTERVAL);

        for node in &config.nodes {
            let url = format!("{}/bird", node.url);
            let resp = client.post(&url).body("show protocols").send().await;

            let status = match resp {
                Ok(r) => match r.text().await {
                    Ok(text) => {
                        let protocols = parser::parse_protocols(&text);

                        let peering = if should_fetch_peering {
                            fetch_peering_info(&client, &node.url).await
                        } else {
                            current_nodes
                                .iter()
                                .find(|n| n.name == node.name)
                                .and_then(|n| n.peering.clone())
                        };

                        NodeStatus {
                            name: node.name.clone(),
                            protocols,
                            last_updated: Utc::now(),
                            error: None,
                            peering,
                        }
                    }
                    Err(e) => {
                        warn!(node = %node.name, error = ?e, "Failed to read BIRD response");
                        let existing = current_nodes.iter().find(|n| n.name == node.name);
                        NodeStatus {
                            name: node.name.clone(),
                            protocols: existing.map(|n| n.protocols.clone()).unwrap_or_default(),
                            last_updated: Utc::now(),
                            error: Some(
                                "Received invalid response from node. Showing cached data.".into(),
                            ),
                            peering: existing.and_then(|n| n.peering.clone()),
                        }
                    }
                },
                Err(e) => {
                    warn!(node = %node.name, error = ?e, "Failed to contact node");
                    let existing = current_nodes.iter().find(|n| n.name == node.name);
                    NodeStatus {
                        name: node.name.clone(),
                        protocols: existing.map(|n| n.protocols.clone()).unwrap_or_default(),
                        last_updated: Utc::now(),
                        error: Some("Unable to reach node. Showing cached data.".into()),
                        peering: existing.and_then(|n| n.peering.clone()),
                    }
                }
            };

            new_statuses.push(status);
        }

        {
            let mut w = state.nodes.write().unwrap();
            *w = new_statuses;
        }

        poll_counter = poll_counter.wrapping_add(1);
        sleep(Duration::from_secs(10)).await;
    }
}

async fn fetch_peering_info(client: &reqwest::Client, node_url: &str) -> Option<PeeringInfo> {
    let peering_url = format!("{}/peering", node_url);

    match client.get(&peering_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Option<PeeringInfo>>().await {
            Ok(peering) => peering,
            Err(e) => {
                warn!(url = %peering_url, error = ?e, "Failed to parse peering info");
                None
            }
        },
        Ok(resp) => {
            warn!(url = %peering_url, status = %resp.status(), "Peering endpoint returned non-success status");
            None
        }
        Err(e) => {
            warn!(url = %peering_url, error = ?e, "Failed to fetch peering info");
            None
        }
    }
}
