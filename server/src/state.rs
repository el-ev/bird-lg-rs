use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::config::PeeringInfo;
pub use common::models::NodeStatus;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<NodeStatus>>>,
    pub peering: Arc<RwLock<HashMap<String, PeeringInfo>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
            peering: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
