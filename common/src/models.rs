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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeProtocol {
    pub name: String,
    pub protocols: Vec<Protocol>,
    pub last_updated: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkInfo {
    pub name: String,
    pub asn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub peering: HashMap<String, PeeringInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WireGuardPeer {
    pub name: String,
    // pub public_key: String,
    // pub endpoint: Option<String>,
    pub latest_handshake: Option<String>,
    pub transfer_rx: String,
    pub transfer_tx: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeWireGuard {
    pub name: String,
    pub peers: Vec<WireGuardPeer>,
    pub last_updated: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "o")]
pub enum DiffOp {
    #[serde(rename = "e")]
    Equal { c: usize },
    #[serde(rename = "i")]
    Insert { i: Vec<Protocol> },
    #[serde(rename = "d")]
    Delete { c: usize },
    #[serde(rename = "r")]
    Replace { i: Vec<Protocol> },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeStatusDiff {
    pub n: String,
    pub d: Vec<DiffOp>,
    pub u: DateTime<Utc>,
    pub e: Option<String>,
}
