use std::{collections::HashMap, rc::Rc};

use chrono::{DateTime, Utc};
use common::{
    api::AppRequest,
    models::{DiffOp, NetworkInfo, NodeProtocol, NodeStatusDiff, NodeWireGuard, PeeringInfo},
};
use yew::prelude::*;

use super::{
    modal::{ModalAction, ModalState},
    traceroute::{TracerouteAction, TracerouteState},
};

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
    pub network_info: Option<NetworkInfo>,
    pub username: String,
    pub backend_url: String,
    pub ws_sender: Option<Callback<AppRequest>>,
}

pub enum Action {
    SetNodes(Vec<NodeProtocol>),
    SetWireGuard(Vec<NodeWireGuard>),
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

impl Reducible for LgState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();

        match action {
            Action::SetNodes(nodes) => {
                next_state.nodes = nodes;
                next_state.data_ready = true;
                next_state.error = None;
            }
            Action::SetWireGuard(wireguard) => {
                next_state.wireguard = wireguard;
            }
            Action::SetError(err) => {
                next_state.error = Some(err);
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
                    // NoChange implies no error
                    node.last_updated = ts;
                }
            }
            Action::ApplyDiff(diffs) => {
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
