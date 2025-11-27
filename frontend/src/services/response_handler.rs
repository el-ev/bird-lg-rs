use crate::store::traceroute::TracerouteAction;
use crate::store::{Action, AppState, NodeTracerouteResult};
use crate::utils::log_error;
use common::models::AppResponse;
use yew::prelude::*;

pub fn handle_app_response(response: AppResponse, state: &UseReducerHandle<AppState>) {
    match response {
        AppResponse::Protocols { data } => {
            state.dispatch(Action::SetNodes(data));
        }
        AppResponse::NoChange { last_updated } => {
            state.dispatch(Action::UpdateTimestamp(last_updated));
        }
        AppResponse::TracerouteInit { node } => {
            state.dispatch(Action::Traceroute(TracerouteAction::InitResult(node)));
        }
        AppResponse::TracerouteUpdate { node, hop } => {
            state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                node,
                NodeTracerouteResult::Hops(vec![hop]),
            )));
        }
        AppResponse::TracerouteError { node, error } => {
            state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                node,
                NodeTracerouteResult::Error(error),
            )));
        }
        AppResponse::RouteLookupInit { node: _ } => {
            state.dispatch(Action::RouteLookupInit(String::new()));
        }
        AppResponse::RouteLookupUpdate { node: _, line } => {
            state.dispatch(Action::RouteLookupUpdate(line));
        }
        AppResponse::ProtocolDetailsInit {
            node: _,
            protocol: _,
        } => {
            state.dispatch(Action::ProtocolDetailsInit(String::new()));
        }
        AppResponse::ProtocolDetailsUpdate {
            node: _,
            protocol: _,
            line,
        } => {
            state.dispatch(Action::ProtocolDetailsUpdate(line));
        }
        AppResponse::NetworkInfo(info) => {
            state.dispatch(Action::SetNetworkInfo(info));
        }
        AppResponse::Error(e) => {
            log_error(&format!("AppResponse Error: {}", e));
        }
    }
}
