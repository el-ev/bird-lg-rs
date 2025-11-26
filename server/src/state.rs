use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::sync::broadcast;

use tracing::warn;

use crate::config::PeeringInfo;
pub use common::models::NodeStatus;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<NodeStatus>>>,
    pub peering: Arc<RwLock<HashMap<String, PeeringInfo>>>,

    pub http_client: reqwest::Client,
    pub tx: broadcast::Sender<Vec<NodeStatus>>,
}

impl AppState {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|e| {
                warn!(error = ?e, "Failed to build HTTP client with config, using defaults");
                reqwest::Client::new()
            });

        let (tx, _) = broadcast::channel(16);

        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
            peering: Arc::new(RwLock::new(HashMap::new())),
            http_client: client,
            tx,
        }
    }
}
