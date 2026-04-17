use std::{collections::HashMap, rc::Rc};

use chrono::{DateTime, Utc};
use common::{
    api::{AppRequest, AppResponse},
    models::{DiffOp, NetworkInfo, NodeProtocol, NodeStatusDiff, NodeWireGuard, PeeringInfo},
};
use yew::prelude::*;

use super::{
    command_output::{CommandOutputKind, CommandOutputState},
    ping::{PingResult, PingState},
    traceroute::{TracerouteResult, TracerouteState},
};

pub type LgStateHandle = UseReducerHandle<LgState>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum WebSocketStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    PollingFallback,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LgState {
    pub nodes: Vec<NodeProtocol>,
    pub wireguard: Vec<NodeWireGuard>,
    pub peering: HashMap<String, PeeringInfo>,
    pub command_output: CommandOutputState,
    pub error: Option<String>,
    pub data_ready: bool,
    pub config_ready: bool,
    pub traceroute: TracerouteState,
    pub ping: PingState,
    pub network_info: Option<NetworkInfo>,
    pub username: String,
    pub backend_url: String,
    pub websocket_status: WebSocketStatus,
    pub ws_sender: Option<Callback<AppRequest>>,
}

impl LgState {
    pub fn is_ws_connected(&self) -> bool {
        matches!(self.websocket_status, WebSocketStatus::Connected) && self.ws_sender.is_some()
    }

    fn set_nodes(&mut self, nodes: Vec<NodeProtocol>) {
        self.nodes = nodes;
        self.data_ready = true;
        self.error = None;
    }

    fn update_timestamp(&mut self, timestamp: DateTime<Utc>) {
        for node in &mut self.nodes {
            node.last_updated = timestamp;
        }
    }

    fn apply_diffs(&mut self, diffs: Vec<NodeStatusDiff>) {
        for diff in diffs {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.name == diff.n) {
                node.error = diff.e;
                if node.error.is_none() {
                    node.last_updated = diff.u;
                }

                let mut new = Vec::new();
                let mut old_idx = 0;

                for op in diff.d {
                    match op {
                        DiffOp::Equal { c: count } => {
                            if old_idx + count <= node.protocols.len() {
                                new.extend_from_slice(&node.protocols[old_idx..old_idx + count]);
                                old_idx += count;
                            }
                        }
                        DiffOp::Insert { i: items } => {
                            new.extend(items);
                        }
                        DiffOp::Delete { c: count } => {
                            old_idx += count;
                        }
                        DiffOp::Replace { i: items } => {
                            new.extend(items.clone());
                            old_idx += items.len();
                        }
                    }
                }
                node.protocols = new;
            }
        }
    }

    fn apply_response(&mut self, response: AppResponse) {
        match response {
            AppResponse::Protocols { data } => self.set_nodes(data),
            AppResponse::NoChange { last_updated } => self.update_timestamp(last_updated),
            AppResponse::ProtocolsDiff { data } => self.apply_diffs(data),
            AppResponse::TracerouteInit { request_id, node } => {
                self.traceroute.initialize(&request_id, node);
            }
            AppResponse::TracerouteUpdate {
                request_id,
                node,
                hops,
            } => {
                self.traceroute
                    .update(&request_id, node, TracerouteResult::Hops(hops));
            }
            AppResponse::TracerouteDone { request_id, .. } => {
                self.traceroute.finish_one(&request_id);
            }
            AppResponse::TracerouteError {
                request_id,
                node,
                error,
            } => {
                self.traceroute
                    .update(&request_id, node, TracerouteResult::Error(error));
                self.traceroute.finish_one(&request_id);
            }
            AppResponse::PingInit { request_id, node } => {
                self.ping.initialize(&request_id, node);
                self.command_output.initialize(&request_id);
            }
            AppResponse::PingUpdate {
                request_id,
                node,
                lines,
            } => {
                self.ping
                    .update(&request_id, node, PingResult::Lines(lines.clone()));
                self.command_output.append_lines(&request_id, &lines);
            }
            AppResponse::PingDone { request_id, .. } => {
                self.ping.finish(&request_id);
                self.command_output.finish(&request_id);
            }
            AppResponse::PingError {
                request_id,
                node,
                error,
            } => {
                self.ping
                    .update(&request_id, node, PingResult::Error(error.clone()));
                self.ping.finish(&request_id);
                self.command_output.append_error(&request_id, &error);
            }
            AppResponse::RouteLookupInit { request_id, .. }
            | AppResponse::ProtocolDetailsInit { request_id, .. } => {
                self.command_output.initialize(&request_id);
            }
            AppResponse::RouteLookupUpdate {
                request_id, lines, ..
            }
            | AppResponse::ProtocolDetailsUpdate {
                request_id, lines, ..
            } => {
                self.command_output.append_lines(&request_id, &lines);
            }
            AppResponse::RouteLookupDone { request_id, .. }
            | AppResponse::ProtocolDetailsDone { request_id, .. } => {
                self.command_output.finish(&request_id);
            }
            AppResponse::RouteLookupError {
                request_id, error, ..
            }
            | AppResponse::ProtocolDetailsError {
                request_id, error, ..
            } => {
                self.command_output.append_error(&request_id, &error);
            }
            AppResponse::WireGuard { data } => {
                self.wireguard = data;
            }
            AppResponse::NetworkInfo(info) => {
                self.network_info = Some(info);
            }
            AppResponse::Error(error) => {
                tracing::error!("AppResponse Error: {}", error);
                self.error = Some(error);
            }
        }
    }
}

pub enum AppEvent {
    SetError(String),
    SetConfig {
        username: String,
        backend_url: String,
    },
    SetWsConnecting,
    SetWsConnected(Callback<AppRequest>),
    SetWsDisconnected,
    SetWsPollingFallback,
    ApplyResponse(AppResponse),
    StartPing {
        request_id: String,
        node: String,
        target: String,
        version: String,
        command: String,
    },
    StartTraceroute {
        request_id: String,
        target: String,
        version: String,
        pending: usize,
    },
    StartCommandOutput {
        request_id: String,
        kind: CommandOutputKind,
        command: String,
    },
    CloseActiveCommandOutput,
}

impl Reducible for LgState {
    type Action = AppEvent;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();

        match action {
            AppEvent::SetError(err) => {
                next_state.error = Some(err);
            }
            AppEvent::SetConfig {
                username,
                backend_url,
            } => {
                next_state.username = username;
                next_state.backend_url = backend_url;
                next_state.config_ready = true;
            }
            AppEvent::SetWsConnecting => {
                next_state.websocket_status = WebSocketStatus::Connecting;
                next_state.ws_sender = None;
            }
            AppEvent::SetWsConnected(sender) => {
                next_state.websocket_status = WebSocketStatus::Connected;
                next_state.ws_sender = Some(sender);
            }
            AppEvent::SetWsDisconnected => {
                next_state.websocket_status = WebSocketStatus::Disconnected;
                next_state.ws_sender = None;
            }
            AppEvent::SetWsPollingFallback => {
                next_state.websocket_status = WebSocketStatus::PollingFallback;
                next_state.ws_sender = None;
            }
            AppEvent::ApplyResponse(response) => {
                next_state.apply_response(response);
            }
            AppEvent::StartPing {
                request_id,
                node,
                target,
                version,
                command,
            } => {
                next_state
                    .ping
                    .start(request_id.clone(), node, target, version);
                next_state
                    .command_output
                    .start(request_id, CommandOutputKind::Ping, command);
            }
            AppEvent::StartTraceroute {
                request_id,
                target,
                version,
                pending,
            } => {
                next_state
                    .traceroute
                    .start(request_id, target, version, pending);
            }
            AppEvent::StartCommandOutput {
                request_id,
                kind,
                command,
            } => {
                next_state.command_output.start(request_id, kind, command);
            }
            AppEvent::CloseActiveCommandOutput => {
                next_state.command_output.close_active();
            }
        }

        Rc::new(next_state)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use common::api::{AppRequest, AppResponse};
    use yew::{Callback, Reducible};

    use super::{AppEvent, LgState, WebSocketStatus};
    use crate::store::{TracerouteResult, command_output::CommandOutputKind};

    #[test]
    fn ping_output_updates_do_not_insert_blank_lines_between_batches() {
        let state = Rc::new(LgState::default());

        let state = state.reduce(AppEvent::StartPing {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            target: "1.1.1.1".to_string(),
            version: "4".to_string(),
            command: "tester@node-1$ ping -c 5 -4 1.1.1.1".to_string(),
        });

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::PingInit {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
        }));

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::PingUpdate {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            lines: vec!["64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=1.09 ms".to_string()],
        }));

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::PingUpdate {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            lines: vec!["64 bytes from 1.1.1.1: icmp_seq=2 ttl=57 time=1.10 ms".to_string()],
        }));

        assert_eq!(
            state
                .command_output
                .active_session()
                .map(|session| session.content.as_str()),
            Some(
                "64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=1.09 ms\n64 bytes from 1.1.1.1: icmp_seq=2 ttl=57 time=1.10 ms\n"
            )
        );
    }

    #[test]
    fn closed_output_session_ignores_late_updates() {
        let state = Rc::new(LgState::default());

        let state = state.reduce(AppEvent::StartCommandOutput {
            request_id: "req-1".to_string(),
            kind: CommandOutputKind::RouteLookup,
            command: "tester@node-1$ birdc show route 1.1.1.0/24".to_string(),
        });
        let state = state.reduce(AppEvent::CloseActiveCommandOutput);
        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::RouteLookupUpdate {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            lines: vec!["should be ignored".to_string()],
        }));

        assert!(state.command_output.active_session().is_none());
        assert!(state.command_output.sessions.is_empty());
    }

    #[test]
    fn websocket_status_tracks_real_connection_lifecycle() {
        let state = Rc::new(LgState::default());
        assert_eq!(state.websocket_status, WebSocketStatus::Disconnected);
        assert!(!state.is_ws_connected());

        let state = state.reduce(AppEvent::SetWsConnecting);
        assert_eq!(state.websocket_status, WebSocketStatus::Connecting);
        assert!(state.ws_sender.is_none());
        assert!(!state.is_ws_connected());

        let sender = Callback::from(|_: AppRequest| ());
        let state = state.reduce(AppEvent::SetWsConnected(sender));
        assert_eq!(state.websocket_status, WebSocketStatus::Connected);
        assert!(state.ws_sender.is_some());
        assert!(state.is_ws_connected());

        let state = state.reduce(AppEvent::SetWsDisconnected);
        assert_eq!(state.websocket_status, WebSocketStatus::Disconnected);
        assert!(state.ws_sender.is_none());
        assert!(!state.is_ws_connected());

        let state = state.reduce(AppEvent::SetWsPollingFallback);
        assert_eq!(state.websocket_status, WebSocketStatus::PollingFallback);
        assert!(state.ws_sender.is_none());
        assert!(!state.is_ws_connected());
    }

    #[test]
    fn apply_response_traceroute_error_updates_session_and_completes_request() {
        let state = Rc::new(LgState::default());

        let state = state.reduce(AppEvent::StartTraceroute {
            request_id: "req-1".to_string(),
            target: "1.1.1.1".to_string(),
            version: "4".to_string(),
            pending: 1,
        });

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::TracerouteInit {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
        }));

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::TracerouteError {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            error: "timeout".to_string(),
        }));

        let session = state
            .traceroute
            .active_session()
            .expect("active traceroute session");
        assert!(session.complete);
        assert_eq!(session.pending, 0);
        assert!(matches!(
            session.results.as_slice(),
            [(node_name, TracerouteResult::Error(message))]
                if node_name == "node-1" && message == "timeout"
        ));
    }

    #[test]
    fn apply_response_route_lookup_update_appends_output_lines() {
        let state = Rc::new(LgState::default());

        let state = state.reduce(AppEvent::StartCommandOutput {
            request_id: "req-1".to_string(),
            kind: CommandOutputKind::RouteLookup,
            command: "tester@node-1$ birdc show route 1.1.1.0/24".to_string(),
        });

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::RouteLookupInit {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
        }));

        let state = state.reduce(AppEvent::ApplyResponse(AppResponse::RouteLookupUpdate {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            lines: vec!["route line".to_string()],
        }));

        assert_eq!(
            state
                .command_output
                .active_session()
                .map(|session| session.content.as_str()),
            Some("route line\n")
        );
    }
}
