use common::api::AppResponse;

use crate::store::{
    AppEvent, LgStateHandle, TracerouteResult,
    ping::{PingAction, PingResult},
    traceroute::TracerouteAction,
};

pub fn handle_app_response(response: AppResponse, state: &LgStateHandle) {
    let events = map_response_to_events(response);
    tracing::debug!(event_count = events.len(), "Dispatching mapped app events");
    for event in events {
        state.dispatch(event);
    }
}

pub fn map_response_to_events(response: AppResponse) -> Vec<AppEvent> {
    match response {
        AppResponse::Protocols { data } => vec![AppEvent::SetNodes(data)],
        AppResponse::NoChange { last_updated } => vec![AppEvent::UpdateTimestamp(last_updated)],
        AppResponse::ProtocolsDiff { data } => vec![AppEvent::ApplyDiff(data)],
        AppResponse::TracerouteInit { node } => {
            vec![AppEvent::Traceroute(TracerouteAction::InitResult(node))]
        }
        AppResponse::TracerouteUpdate { node, hops } => vec![AppEvent::Traceroute(
            TracerouteAction::UpdateResult(node, TracerouteResult::Hops(hops)),
        )],
        AppResponse::TracerouteError { node, error } => vec![AppEvent::Traceroute(
            TracerouteAction::UpdateResult(node, TracerouteResult::Error(error)),
        )],
        AppResponse::PingInit { node } => vec![
            AppEvent::Ping(PingAction::InitResult(node)),
            AppEvent::PingModalInit,
        ],
        AppResponse::PingUpdate { node, lines } => {
            let result = PingResult::Lines(lines);
            vec![
                AppEvent::Ping(PingAction::UpdateResult(node.clone(), result.clone())),
                AppEvent::PingModalUpdate { node, result },
            ]
        }
        AppResponse::PingError { node, error } => {
            let result = PingResult::Error(error);
            vec![
                AppEvent::Ping(PingAction::UpdateResult(node.clone(), result.clone())),
                AppEvent::PingModalUpdate { node, result },
            ]
        }
        AppResponse::RouteLookupInit { node: _ } => vec![AppEvent::RouteLookupInit],
        AppResponse::RouteLookupUpdate { node, lines } => {
            vec![AppEvent::RouteLookupUpdate { node, lines }]
        }
        AppResponse::ProtocolDetailsInit {
            node: _,
            protocol: _,
        } => vec![AppEvent::ProtocolDetailsInit],
        AppResponse::ProtocolDetailsUpdate {
            node,
            protocol,
            lines,
        } => vec![AppEvent::ProtocolDetailsUpdate {
            node,
            protocol,
            lines,
        }],
        AppResponse::WireGuard { data } => vec![AppEvent::SetWireGuard(data)],
        AppResponse::NetworkInfo(info) => vec![AppEvent::SetNetworkInfo(info)],
        AppResponse::Error(e) => {
            tracing::error!("AppResponse Error: {}", e);
            Vec::new()
        }
    }
}
