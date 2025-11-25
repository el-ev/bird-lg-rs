use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use crate::config::PeeringInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Protocol {
    pub name: String,
    pub proto: String,
    pub table: String,
    pub state: String,
    pub since: String,
    pub info: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeStatus {
    pub name: String,
    pub protocols: Vec<Protocol>,
    pub last_updated: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<NodeStatus>>>,
    pub peering: Arc<RwLock<std::collections::HashMap<String, PeeringInfo>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
            peering: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}
