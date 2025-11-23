use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Protocol {
    pub name: String,
    pub proto: String,
    pub table: String,
    pub state: String,
    pub since: String,
    pub info: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeStatus {
    pub name: String,
    pub protocols: Vec<Protocol>,
    pub last_updated: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TracerouteHop {
    pub hop: u32,
    pub address: Option<String>,
    pub hostname: Option<String>,
    pub rtts: Option<Vec<f32>>,
}
