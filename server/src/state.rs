use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use tokio::sync::broadcast;

use tracing::warn;

use crate::config::PeeringInfo;
pub use common::models::{AppRequest, NodeStatus, AppResponse};

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<NodeStatus>>>,
    pub peering: Arc<RwLock<HashMap<String, PeeringInfo>>>,

    pub http_client: reqwest::Client,
    pub tx: broadcast::Sender<AppResponse>,

    pub last_request_time: Arc<RwLock<Option<Instant>>>,
    pub is_polling_active: Arc<AtomicBool>,
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
            last_request_time: Arc::new(RwLock::new(None)),
            is_polling_active: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn record_request(&self) {
        *self.last_request_time.write().unwrap() = Some(Instant::now());
        self.is_polling_active.store(true, Ordering::Relaxed);
    }
}
