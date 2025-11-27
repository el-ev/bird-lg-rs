use std::collections::HashMap;

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
pub struct PeeringInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_local_ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wg_pubkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeStatus {
    pub name: String,
    pub protocols: Vec<Protocol>,
    pub last_updated: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkInfo {
    pub name: String,
    pub asn: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub peering: HashMap<String, PeeringInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TracerouteHop {
    pub hop: u32,
    pub address: Option<String>,
    pub hostname: Option<String>,
    pub rtts: Option<Vec<f32>>,
}

#[derive(Deserialize)]
pub struct TracerouteParams {
    pub target: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum AppRequest {
    GetProtocols,
    Traceroute {
        node: String,
        target: String,
    },
    RouteLookup {
        node: String,
        target: String,
        #[serde(default)]
        all: bool,
    },
    ProtocolDetails {
        node: String,
        protocol: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AppResponse {
    Protocols {
        data: Vec<NodeStatus>,
    },
    NoChange {
        last_updated: DateTime<Utc>,
    },
    TracerouteResult {
        node: String,
        result: String,
    },
    RouteLookupResult {
        node: String,
        result: String,
    },
    ProtocolDetailsResult {
        node: String,
        protocol: String,
        details: String,
    },
    NetworkInfo(NetworkInfo),
    Error(String),
}
