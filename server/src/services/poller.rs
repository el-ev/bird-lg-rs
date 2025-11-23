use std::{sync::Arc, time::Duration};

use chrono::Utc;
use tokio::time::sleep;
use tracing::warn;

use crate::{
    config::Config,
    parser,
    state::{AppState, NodeStatus},
};

/// Periodically polls configured nodes and refreshes shared application state.
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

    loop {
        let mut new_statuses = Vec::new();
        let current_nodes = { state.nodes.read().unwrap().clone() };

        for node in &config.nodes {
            let url = format!("{}/bird", node.url);
            let resp = client.post(&url).body("show protocols").send().await;

            let status = match resp {
                Ok(r) => match r.text().await {
                    Ok(text) => {
                        let protocols = parser::parse_protocols(&text);
                        NodeStatus {
                            name: node.name.clone(),
                            protocols,
                            last_updated: Utc::now(),
                            error: None,
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
                    }
                }
            };

            new_statuses.push(status);
        }

        {
            let mut w = state.nodes.write().unwrap();
            *w = new_statuses;
        }

        sleep(Duration::from_secs(10)).await;
    }
}
