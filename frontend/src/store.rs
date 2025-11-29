use chrono::{DateTime, Utc};
use common::models::{DiffOp, NetworkInfo, NodeStatus, NodeStatusDiff, PeeringInfo};
use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

pub mod modal;
pub mod traceroute;

use modal::{ModalAction, ModalState};
use traceroute::{TracerouteAction, TracerouteState};

pub use traceroute::NodeTracerouteResult;

use common::api::AppRequest;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AppState {
    pub nodes: Vec<NodeStatus>,
    pub peering: HashMap<String, PeeringInfo>,
    pub modal: ModalState,
    pub fetch_error: Option<String>,
    pub data_ready: bool,
    pub config_ready: bool,
    pub traceroute: TracerouteState,
    pub network_info: Option<NetworkInfo>,
    pub username: String,
    pub backend_url: String,
    pub ws_sender: Option<Callback<AppRequest>>,
}

pub enum Action {
    SetNodes(Vec<NodeStatus>),
    SetError(String),
    Modal(ModalAction),
    Traceroute(TracerouteAction),
    SetNetworkInfo(NetworkInfo),
    SetConfig {
        username: String,
        backend_url: String,
    },
    SetWsSender(Callback<AppRequest>),
    ClearWsSender,
    UpdateTimestamp(DateTime<Utc>),
    ApplyDiff(Vec<NodeStatusDiff>),
    RouteLookupInit(String),
    RouteLookupUpdate(Vec<String>),
    ProtocolDetailsInit(String),
    ProtocolDetailsUpdate(Vec<String>),
}

impl Reducible for AppState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();

        match action {
            Action::SetNodes(nodes) => {
                next_state.nodes = nodes;
                next_state.data_ready = true;
                next_state.fetch_error = None;
            }
            Action::SetError(err) => {
                next_state.fetch_error = Some(err);
            }
            Action::Modal(act) => {
                next_state.modal.reduce(act);
            }
            Action::Traceroute(act) => {
                next_state.traceroute.reduce(act);
            }
            Action::SetNetworkInfo(info) => {
                next_state.network_info = Some(info);
            }
            Action::SetConfig {
                username,
                backend_url,
            } => {
                next_state.username = username;
                next_state.backend_url = backend_url;
                next_state.config_ready = true;
            }
            Action::SetWsSender(sender) => {
                next_state.ws_sender = Some(sender);
            }
            Action::ClearWsSender => {
                next_state.ws_sender = None;
            }
            Action::UpdateTimestamp(ts) => {
                for node in &mut next_state.nodes {
                    if node.error.is_none() {
                        node.last_updated = ts;
                    }
                }
            }
            Action::ApplyDiff(diffs) => {
                for diff in diffs {
                    if let Some(node) = next_state.nodes.iter_mut().find(|n| n.name == diff.n) {
                        node.last_updated = diff.u;
                        node.error = diff.e;

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
            Action::RouteLookupInit(result) => {
                next_state.modal.content = result;
            }
            Action::RouteLookupUpdate(lines) => {
                next_state.modal.content = self.modal.content.clone() + "\n" + &lines.join("\n");
            }
            Action::ProtocolDetailsInit(result) => {
                next_state.modal.content = result;
            }
            Action::ProtocolDetailsUpdate(lines) => {
                next_state.modal.content = self.modal.content.clone() + "\n" + &lines.join("\n");
            }
        }

        Rc::new(next_state)
    }
}
