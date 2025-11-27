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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum HopRange {
    Single(u32),
    Range(u32, u32),
}

impl HopRange {
    pub fn start(&self) -> u32 {
        match self {
            HopRange::Single(n) | HopRange::Range(n, _) => *n,
        }
    }

    pub fn end(&self) -> u32 {
        match self {
            HopRange::Single(n) | HopRange::Range(_, n) => *n,
        }
    }
}

impl std::fmt::Display for HopRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HopRange::Single(n) => write!(f, "{}", n),
            HopRange::Range(start, end) => write!(f, "{}-{}", start, end),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TracerouteHop {
    pub hop: HopRange,
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
    #[serde(rename = "gp")]
    GetProtocols,
    #[serde(rename = "tr")]
    Traceroute { node: String, target: String },
    #[serde(rename = "rl")]
    RouteLookup {
        node: String,
        target: String,
        #[serde(default)]
        all: bool,
    },
    #[serde(rename = "pd")]
    ProtocolDetails { node: String, protocol: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum AppResponse {
    #[serde(rename = "pr")]
    Protocols { data: Vec<NodeStatus> },
    #[serde(rename = "nc")]
    NoChange { last_updated: DateTime<Utc> },
    #[serde(rename = "tri")]
    TracerouteInit { node: String },
    #[serde(rename = "tru")]
    TracerouteUpdate { node: String, hop: TracerouteHop },
    #[serde(rename = "tre")]
    TracerouteError { node: String, error: String },
    #[serde(rename = "rli")]
    RouteLookupInit { node: String },
    // TODO send batch lines
    #[serde(rename = "rlu")]
    RouteLookupUpdate { node: String, line: String },
    #[serde(rename = "pdi")]
    ProtocolDetailsInit { node: String, protocol: String },
    #[serde(rename = "pdu")]
    ProtocolDetailsUpdate {
        node: String,
        protocol: String,
        line: String,
    },
    #[serde(rename = "ni")]
    NetworkInfo(NetworkInfo),
    #[serde(rename = "e")]
    Error(String),
}
