use std::{collections::HashMap, rc::Rc};

use chrono::{DateTime, Utc};
use common::{
    api::AppRequest,
    models::{DiffOp, NetworkInfo, NodeProtocol, NodeStatusDiff, NodeWireGuard, PeeringInfo},
};
use yew::prelude::*;

use super::{
    modal::{ModalAction, ModalState},
    ping::{PingAction, PingResult, PingState},
    traceroute::{TracerouteAction, TracerouteState},
};
// ... (skip lines)
pub type LgStateHandle = UseReducerHandle<LgState>;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LgState {
    pub nodes: Vec<NodeProtocol>,
    pub wireguard: Vec<NodeWireGuard>,
    pub peering: HashMap<String, PeeringInfo>,
    pub modal: ModalState,
    pub error: Option<String>,
    pub data_ready: bool,
    pub config_ready: bool,
    pub traceroute: TracerouteState,
    pub ping: PingState,
    pub network_info: Option<NetworkInfo>,
    pub username: String,
    pub backend_url: String,
    pub ws_sender: Option<Callback<AppRequest>>,
    pub route_lookup_context: Option<RouteLookupContext>,
    pub protocol_details_context: Option<ProtocolDetailsContext>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouteLookupContext {
    pub node: String,
    pub target: String,
    pub all: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtocolDetailsContext {
    pub node: String,
    pub protocol: String,
}

pub enum AppEvent {
    SetNodes(Vec<NodeProtocol>),
    SetWireGuard(Vec<NodeWireGuard>),
    SetError(String),
    Modal(ModalAction),
    Traceroute(TracerouteAction),
    Ping(PingAction),
    SetNetworkInfo(NetworkInfo),
    SetConfig {
        username: String,
        backend_url: String,
    },
    SetWsSender(Callback<AppRequest>),
    ClearWsSender,
    UpdateTimestamp(DateTime<Utc>),
    ApplyDiff(Vec<NodeStatusDiff>),
    SetRouteLookupContext(RouteLookupContext),
    RouteLookupInit,
    RouteLookupUpdate {
        node: String,
        lines: Vec<String>,
    },
    SetProtocolDetailsContext(ProtocolDetailsContext),
    ProtocolDetailsInit,
    ProtocolDetailsUpdate {
        node: String,
        protocol: String,
        lines: Vec<String>,
    },
    PingModalInit,
    PingModalUpdate {
        node: String,
        result: PingResult,
    },
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
            AppEvent::Modal(act) => {
                next_state.modal.reduce(act);
            }
            AppEvent::Traceroute(act) => {
                next_state.traceroute.reduce(act);
            }
            AppEvent::Ping(act) => {
                next_state.ping.reduce(act);
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
                    // NoChange implies no error
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
            AppEvent::SetRouteLookupContext(ctx) => {
                next_state.route_lookup_context = Some(ctx);
            }
            AppEvent::RouteLookupInit => {
                next_state.modal.content = String::new();
            }
            AppEvent::RouteLookupUpdate { node, lines } => {
                if let Some(ctx) = &mut next_state.route_lookup_context {
                    ctx.node = node;
                }
                append_modal_lines(&mut next_state.modal.content, &lines);
            }
            AppEvent::SetProtocolDetailsContext(ctx) => {
                next_state.protocol_details_context = Some(ctx);
            }
            AppEvent::ProtocolDetailsInit => {
                next_state.modal.content = String::new();
            }
            AppEvent::ProtocolDetailsUpdate {
                node,
                protocol,
                lines,
            } => {
                if let Some(ctx) = &mut next_state.protocol_details_context {
                    ctx.node = node;
                    ctx.protocol = protocol;
                }
                append_modal_lines(&mut next_state.modal.content, &lines);
            }
            AppEvent::PingModalInit => {
                next_state.modal.content = String::new();
            }
            AppEvent::PingModalUpdate { node, result } => match result {
                PingResult::Lines(lines) => {
                    next_state.ping.node = node;
                    append_modal_lines(&mut next_state.modal.content, &lines);
                }
                PingResult::Error(err) => {
                    next_state.ping.node = node;
                    if !next_state.modal.content.is_empty() {
                        next_state.modal.content.push('\n');
                    }
                    next_state.modal.content.push_str("Error: ");
                    next_state.modal.content.push_str(&err);
                    next_state.modal.content.push('\n');
                }
            },
        }

        Rc::new(next_state)
    }
}

fn append_modal_lines(content: &mut String, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(&lines.join("\n"));
    content.push('\n');
}
