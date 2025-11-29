use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{NetworkInfo, NodeStatus, NodeStatusDiff};
use crate::traceroute::TracerouteHop;

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
    #[serde(rename = "pd")]
    ProtocolsDiff { data: Vec<NodeStatusDiff> },
    #[serde(rename = "nc")]
    NoChange(DateTime<Utc>),
    #[serde(rename = "tri")]
    TracerouteInit { node: String },
    #[serde(rename = "tru")]
    TracerouteUpdate {
        node: String,
        hops: Vec<TracerouteHop>,
    },
    #[serde(rename = "tre")]
    TracerouteError { node: String, error: String },
    #[serde(rename = "rli")]
    RouteLookupInit { node: String },
    #[serde(rename = "rlu")]
    RouteLookupUpdate { node: String, lines: Vec<String> },
    #[serde(rename = "pdi")]
    ProtocolDetailsInit { node: String, protocol: String },
    #[serde(rename = "pdu")]
    ProtocolDetailsUpdate {
        node: String,
        protocol: String,
        lines: Vec<String>,
    },
    #[serde(rename = "ni")]
    NetworkInfo(NetworkInfo),
    #[serde(rename = "e")]
    Error(String),
}
