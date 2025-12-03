use common::api::AppResponse;

use crate::store::{Action, LgStateHandle, TracerouteResult, traceroute::TracerouteAction};

pub fn handle_app_response(response: AppResponse, state: &LgStateHandle) {
    match response {
        AppResponse::Protocols { data } => {
            state.dispatch(Action::SetNodes(data));
        }
        AppResponse::NoChange { last_updated } => {
            state.dispatch(Action::UpdateTimestamp(last_updated));
        }
        AppResponse::ProtocolsDiff { data } => {
            state.dispatch(Action::ApplyDiff(data));
        }
        AppResponse::TracerouteInit { node } => {
            state.dispatch(Action::Traceroute(TracerouteAction::InitResult(node)));
        }
        AppResponse::TracerouteUpdate { node, hops } => {
            state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                node,
                TracerouteResult::Hops(hops),
            )));
        }
        AppResponse::TracerouteError { node, error } => {
            state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                node,
                TracerouteResult::Error(error),
            )));
        }
        AppResponse::RouteLookupInit { node: _ } => {
            state.dispatch(Action::RouteLookupInit(String::new()));
        }
        AppResponse::RouteLookupUpdate { node: _, lines } => {
            state.dispatch(Action::RouteLookupUpdate(lines));
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
            lines,
        } => {
            state.dispatch(Action::ProtocolDetailsUpdate(lines));
        }
        AppResponse::WireGuard { data } => {
            state.dispatch(Action::SetWireGuard(data));
        }
        AppResponse::NetworkInfo(info) => {
            state.dispatch(Action::SetNetworkInfo(info));
        }
        AppResponse::Error(e) => {
            tracing::error!("AppResponse Error: {}", e);
        }
    }
}
