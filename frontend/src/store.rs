use std::rc::Rc;
use yew::prelude::*;

use crate::models::{NetworkInfo, NodeStatus};

pub mod modal;
pub mod traceroute;

use modal::{ModalAction, ModalState};
use traceroute::{TracerouteAction, TracerouteState};

pub use traceroute::NodeTracerouteResult;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AppState {
    pub nodes: Vec<NodeStatus>,
    pub modal: ModalState,
    pub fetch_error: Option<String>,
    pub data_ready: bool,
    pub config_ready: bool,
    pub traceroute: TracerouteState,
    pub network_info: Option<NetworkInfo>,
}

pub enum Action {
    SetNodes(Vec<NodeStatus>),
    SetError(String),
    ClearError,
    SetDataReady(bool),
    SetConfigReady(bool),
    Modal(ModalAction),
    Traceroute(TracerouteAction),
    SetNetworkInfo(Option<NetworkInfo>),
}

impl Reducible for AppState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();

        match action {
            Action::SetNodes(nodes) => {
                next_state.nodes = nodes;
            }
            Action::SetError(err) => {
                next_state.fetch_error = Some(err);
            }
            Action::ClearError => {
                next_state.fetch_error = None;
            }
            Action::SetDataReady(ready) => {
                next_state.data_ready = ready;
            }
            Action::SetConfigReady(ready) => {
                next_state.config_ready = ready;
            }
            Action::Modal(act) => {
                next_state.modal.reduce(act);
            }
            Action::Traceroute(act) => {
                next_state.traceroute.reduce(act);
            }
            Action::SetNetworkInfo(info) => {
                next_state.network_info = info;
            }
        }

        Rc::new(next_state)
    }
}
