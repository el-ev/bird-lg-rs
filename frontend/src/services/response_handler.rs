use crate::store::traceroute::TracerouteAction;
use crate::store::{Action, AppState, NodeTracerouteResult};
use crate::utils::{log_error, parse_traceroute_line};
use common::models::{AppResponse, TracerouteHop};
use yew::prelude::*;

pub fn handle_app_response(response: AppResponse, state: &UseReducerHandle<AppState>) {
    match response {
        AppResponse::Protocols { data } => {
            state.dispatch(Action::SetNodes(data));
        }
        AppResponse::NoChange { last_updated } => {
            state.dispatch(Action::UpdateTimestamp(last_updated));
        }
        AppResponse::TracerouteResult { node, result } => {
            let hops: Vec<TracerouteHop> = result
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(parse_traceroute_line)
                .collect();

            state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                node,
                NodeTracerouteResult::Hops(hops),
            )));
        }
        AppResponse::RouteLookupResult { node: _, result } => {
            state.dispatch(Action::RouteLookupResult(result));
        }
        AppResponse::ProtocolDetailsResult {
            node: _,
            protocol: _,
            details,
        } => {
            let filtered = common::utils::filter_protocol_details(&details);
            state.dispatch(Action::ProtocolDetailsResult(filtered));
        }
        AppResponse::NetworkInfo(info) => {
            state.dispatch(Action::SetNetworkInfo(info));
        }
        AppResponse::Error(e) => {
            log_error(&format!("AppResponse Error: {}", e));
        }
    }
}
