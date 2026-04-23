use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    models::{NetworkInfo, NodeProtocol, NodeStatusDiff, NodeWireGuard},
    traceroute::TracerouteHop,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum AppRequest {
    #[serde(rename = "gp")]
    GetProtocols,
    #[serde(rename = "gw")]
    GetWireGuard,
    #[serde(rename = "tr")]
    Traceroute {
        request_id: String,
        node: String,
        target: String,
        #[serde(default)]
        version: String,
    },
    #[serde(rename = "rl")]
    RouteLookup {
        request_id: String,
        node: String,
        target: String,
        #[serde(default)]
        all: bool,
    },
    #[serde(rename = "pd")]
    ProtocolDetails {
        request_id: String,
        node: String,
        protocol: String,
    },
    #[serde(rename = "pi")]
    Ping {
        request_id: String,
        node: String,
        target: String,
        #[serde(default)]
        version: String,
    },
    #[serde(rename = "pr_routes")]
    PeerRoutes {
        request_id: String,
        node: String,
        peer: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "t")]
pub enum AppResponse {
    #[serde(rename = "pr")]
    Protocols { data: Vec<NodeProtocol> },
    #[serde(rename = "pd")]
    ProtocolsDiff { data: Vec<NodeStatusDiff> },
    #[serde(rename = "nc")]
    NoChange { last_updated: DateTime<Utc> },
    #[serde(rename = "wg")]
    WireGuard { data: Vec<NodeWireGuard> },
    #[serde(rename = "tri")]
    TracerouteInit { request_id: String, node: String },
    #[serde(rename = "tru")]
    TracerouteUpdate {
        request_id: String,
        node: String,
        hops: Vec<TracerouteHop>,
    },
    #[serde(rename = "trd")]
    TracerouteDone { request_id: String, node: String },
    #[serde(rename = "tre")]
    TracerouteError {
        request_id: String,
        node: String,
        error: String,
    },
    #[serde(rename = "pii")]
    PingInit { request_id: String, node: String },
    #[serde(rename = "piu")]
    PingUpdate {
        request_id: String,
        node: String,
        lines: Vec<String>,
    },
    #[serde(rename = "pid")]
    PingDone { request_id: String, node: String },
    #[serde(rename = "pie")]
    PingError {
        request_id: String,
        node: String,
        error: String,
    },
    #[serde(rename = "rli")]
    RouteLookupInit { request_id: String, node: String },
    #[serde(rename = "rlu")]
    RouteLookupUpdate {
        request_id: String,
        node: String,
        lines: Vec<String>,
    },
    #[serde(rename = "rld")]
    RouteLookupDone { request_id: String, node: String },
    #[serde(rename = "rle")]
    RouteLookupError {
        request_id: String,
        node: String,
        error: String,
    },
    #[serde(rename = "pdi")]
    ProtocolDetailsInit {
        request_id: String,
        node: String,
        protocol: String,
    },
    #[serde(rename = "pdu")]
    ProtocolDetailsUpdate {
        request_id: String,
        node: String,
        protocol: String,
        lines: Vec<String>,
    },
    #[serde(rename = "pdd")]
    ProtocolDetailsDone {
        request_id: String,
        node: String,
        protocol: String,
    },
    #[serde(rename = "pde")]
    ProtocolDetailsError {
        request_id: String,
        node: String,
        protocol: String,
        error: String,
    },
    #[serde(rename = "ni")]
    NetworkInfo(NetworkInfo),
    #[serde(rename = "e")]
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::{AppRequest, AppResponse};

    fn request_id_for_request(request: &AppRequest) -> Option<&str> {
        match request {
            AppRequest::GetProtocols | AppRequest::GetWireGuard => None,
            AppRequest::Traceroute { request_id, .. }
            | AppRequest::RouteLookup { request_id, .. }
            | AppRequest::ProtocolDetails { request_id, .. }
            | AppRequest::Ping { request_id, .. }
            | AppRequest::PeerRoutes { request_id, .. } => Some(request_id.as_str()),
        }
    }

    fn request_id_for_response(response: &AppResponse) -> Option<&str> {
        match response {
            AppResponse::Protocols { .. }
            | AppResponse::ProtocolsDiff { .. }
            | AppResponse::NoChange { .. }
            | AppResponse::WireGuard { .. }
            | AppResponse::NetworkInfo(_)
            | AppResponse::Error(_) => None,
            AppResponse::TracerouteInit { request_id, .. }
            | AppResponse::TracerouteUpdate { request_id, .. }
            | AppResponse::TracerouteDone { request_id, .. }
            | AppResponse::TracerouteError { request_id, .. }
            | AppResponse::PingInit { request_id, .. }
            | AppResponse::PingUpdate { request_id, .. }
            | AppResponse::PingDone { request_id, .. }
            | AppResponse::PingError { request_id, .. }
            | AppResponse::RouteLookupInit { request_id, .. }
            | AppResponse::RouteLookupUpdate { request_id, .. }
            | AppResponse::RouteLookupDone { request_id, .. }
            | AppResponse::RouteLookupError { request_id, .. }
            | AppResponse::ProtocolDetailsInit { request_id, .. }
            | AppResponse::ProtocolDetailsUpdate { request_id, .. }
            | AppResponse::ProtocolDetailsDone { request_id, .. }
            | AppResponse::ProtocolDetailsError { request_id, .. } => Some(request_id.as_str()),
        }
    }

    #[test]
    fn streamed_requests_round_trip_request_ids() {
        let requests = [
            AppRequest::Traceroute {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                target: "1.1.1.1".to_string(),
                version: "4".to_string(),
            },
            AppRequest::RouteLookup {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                target: "1.1.1.0/24".to_string(),
                all: true,
            },
            AppRequest::ProtocolDetails {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                protocol: "bgp1".to_string(),
            },
            AppRequest::Ping {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                target: "1.1.1.1".to_string(),
                version: "6".to_string(),
            },
            AppRequest::PeerRoutes {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                peer: "bgp_peer1".to_string(),
            },
        ];

        for request in requests {
            let json = serde_json::to_string(&request).expect("request serializes");
            assert!(
                json.contains("\"request_id\":\"req-123\""),
                "serialized request missing request_id: {json}"
            );

            let round_trip: AppRequest = serde_json::from_str(&json).expect("request deserializes");
            assert_eq!(request_id_for_request(&round_trip), Some("req-123"));
        }
    }

    #[test]
    fn streamed_responses_round_trip_request_ids() {
        let responses = [
            AppResponse::TracerouteInit {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
            },
            AppResponse::TracerouteUpdate {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                hops: Vec::new(),
            },
            AppResponse::TracerouteDone {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
            },
            AppResponse::TracerouteError {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                error: "timeout".to_string(),
            },
            AppResponse::PingInit {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
            },
            AppResponse::PingUpdate {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                lines: vec!["pong".to_string()],
            },
            AppResponse::PingDone {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
            },
            AppResponse::PingError {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                error: "timeout".to_string(),
            },
            AppResponse::RouteLookupInit {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
            },
            AppResponse::RouteLookupUpdate {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                lines: vec!["route".to_string()],
            },
            AppResponse::RouteLookupDone {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
            },
            AppResponse::RouteLookupError {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                error: "failure".to_string(),
            },
            AppResponse::ProtocolDetailsInit {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                protocol: "bgp1".to_string(),
            },
            AppResponse::ProtocolDetailsUpdate {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                protocol: "bgp1".to_string(),
                lines: vec!["detail".to_string()],
            },
            AppResponse::ProtocolDetailsDone {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                protocol: "bgp1".to_string(),
            },
            AppResponse::ProtocolDetailsError {
                request_id: "req-123".to_string(),
                node: "edge-a".to_string(),
                protocol: "bgp1".to_string(),
                error: "failure".to_string(),
            },
        ];

        for response in responses {
            let json = serde_json::to_string(&response).expect("response serializes");
            assert!(
                json.contains("\"request_id\":\"req-123\""),
                "serialized response missing request_id: {json}"
            );

            let round_trip: AppResponse =
                serde_json::from_str(&json).expect("response deserializes");
            assert_eq!(request_id_for_response(&round_trip), Some("req-123"));
        }
    }
}
