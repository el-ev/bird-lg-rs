use std::{collections::HashMap, rc::Rc};

use chrono::{DateTime, Utc};
use common::{
    api::AppRequest,
    models::{DiffOp, NetworkInfo, NodeProtocol, NodeStatusDiff, NodeWireGuard, PeeringInfo},
};
use yew::prelude::*;

use super::{
    command_output::{CommandOutputKind, CommandOutputState},
    ping::{PingResult, PingState},
    traceroute::{TracerouteResult, TracerouteState},
};

pub type LgStateHandle = UseReducerHandle<LgState>;

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
    pub ws_sender: Option<Callback<AppRequest>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PingStreamEvent {
    Init {
        request_id: String,
        node: String,
    },
    Update {
        request_id: String,
        node: String,
        lines: Vec<String>,
    },
    Done {
        request_id: String,
    },
    Error {
        request_id: String,
        node: String,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TracerouteStreamEvent {
    Init {
        request_id: String,
        node: String,
    },
    Update {
        request_id: String,
        node: String,
        result: TracerouteResult,
    },
    Done {
        request_id: String,
    },
    Error {
        request_id: String,
        node: String,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandOutputEvent {
    Init {
        request_id: String,
    },
    Update {
        request_id: String,
        lines: Vec<String>,
    },
    Done {
        request_id: String,
    },
    Error {
        request_id: String,
        error: String,
    },
}

pub enum AppEvent {
    SetNodes(Vec<NodeProtocol>),
    SetWireGuard(Vec<NodeWireGuard>),
    SetError(String),
    SetNetworkInfo(NetworkInfo),
    SetConfig {
        username: String,
        backend_url: String,
    },
    SetWsSender(Callback<AppRequest>),
    ClearWsSender,
    UpdateTimestamp(DateTime<Utc>),
    ApplyDiff(Vec<NodeStatusDiff>),
    StartPing {
        request_id: String,
        node: String,
        target: String,
        version: String,
        command: String,
    },
    PingStream(PingStreamEvent),
    StartTraceroute {
        request_id: String,
        target: String,
        version: String,
        pending: usize,
    },
    TracerouteStream(TracerouteStreamEvent),
    StartCommandOutput {
        request_id: String,
        kind: CommandOutputKind,
        command: String,
    },
    CommandOutputStream(CommandOutputEvent),
    CloseActiveCommandOutput,
}

impl Reducible for LgState {
    type Action = AppEvent;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();

        match action {
            AppEvent::SetNodes(nodes) => {
                next_state.nodes = nodes;
                next_state.data_ready = true;
                next_state.error = None;
            }
            AppEvent::SetWireGuard(wireguard) => {
                next_state.wireguard = wireguard;
            }
            AppEvent::SetError(err) => {
                next_state.error = Some(err);
            }
            AppEvent::SetNetworkInfo(info) => {
                next_state.network_info = Some(info);
            }
            AppEvent::SetConfig {
                username,
                backend_url,
            } => {
                next_state.username = username;
                next_state.backend_url = backend_url;
                next_state.config_ready = true;
            }
            AppEvent::SetWsSender(sender) => {
                next_state.ws_sender = Some(sender);
            }
            AppEvent::ClearWsSender => {
                next_state.ws_sender = None;
            }
            AppEvent::UpdateTimestamp(ts) => {
                for node in &mut next_state.nodes {
                    node.last_updated = ts;
                }
            }
            AppEvent::ApplyDiff(diffs) => {
                for diff in diffs {
                    if let Some(node) = next_state.nodes.iter_mut().find(|n| n.name == diff.n) {
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
                                        new.extend_from_slice(
                                            &node.protocols[old_idx..old_idx + count],
                                        );
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
            AppEvent::PingStream(event) => match event {
                PingStreamEvent::Init { request_id, node } => {
                    next_state.ping.initialize(&request_id, node);
                    next_state.command_output.initialize(&request_id);
                }
                PingStreamEvent::Update {
                    request_id,
                    node,
                    lines,
                } => {
                    next_state
                        .ping
                        .update(&request_id, node, PingResult::Lines(lines.clone()));
                    next_state.command_output.append_lines(&request_id, &lines);
                }
                PingStreamEvent::Done { request_id } => {
                    next_state.ping.finish(&request_id);
                    next_state.command_output.finish(&request_id);
                }
                PingStreamEvent::Error {
                    request_id,
                    node,
                    error,
                } => {
                    next_state
                        .ping
                        .update(&request_id, node, PingResult::Error(error.clone()));
                    next_state.ping.finish(&request_id);
                    next_state.command_output.append_error(&request_id, &error);
                }
            },
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
            AppEvent::TracerouteStream(event) => match event {
                TracerouteStreamEvent::Init { request_id, node } => {
                    next_state.traceroute.initialize(&request_id, node);
                }
                TracerouteStreamEvent::Update {
                    request_id,
                    node,
                    result,
                } => {
                    next_state.traceroute.update(&request_id, node, result);
                }
                TracerouteStreamEvent::Done { request_id } => {
                    next_state.traceroute.finish_one(&request_id);
                }
                TracerouteStreamEvent::Error {
                    request_id,
                    node,
                    error,
                } => {
                    next_state
                        .traceroute
                        .update(&request_id, node, TracerouteResult::Error(error));
                    next_state.traceroute.finish_one(&request_id);
                }
            },
            AppEvent::StartCommandOutput {
                request_id,
                kind,
                command,
            } => {
                next_state.command_output.start(request_id, kind, command);
            }
            AppEvent::CommandOutputStream(event) => match event {
                CommandOutputEvent::Init { request_id } => {
                    next_state.command_output.initialize(&request_id);
                }
                CommandOutputEvent::Update { request_id, lines } => {
                    next_state.command_output.append_lines(&request_id, &lines);
                }
                CommandOutputEvent::Done { request_id } => {
                    next_state.command_output.finish(&request_id);
                }
                CommandOutputEvent::Error { request_id, error } => {
                    next_state.command_output.append_error(&request_id, &error);
                }
            },
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

    use yew::Reducible;

    use super::{AppEvent, CommandOutputEvent, LgState, PingStreamEvent};
    use crate::store::command_output::CommandOutputKind;

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

        let state = state.reduce(AppEvent::PingStream(PingStreamEvent::Init {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
        }));

        let state = state.reduce(AppEvent::PingStream(PingStreamEvent::Update {
            request_id: "req-1".to_string(),
            node: "node-1".to_string(),
            lines: vec!["64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=1.09 ms".to_string()],
        }));

        let state = state.reduce(AppEvent::PingStream(PingStreamEvent::Update {
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
        let state = state.reduce(AppEvent::CommandOutputStream(CommandOutputEvent::Update {
            request_id: "req-1".to_string(),
            lines: vec!["should be ignored".to_string()],
        }));

        assert!(state.command_output.active_session().is_none());
        assert!(state.command_output.sessions.is_empty());
    }
}
