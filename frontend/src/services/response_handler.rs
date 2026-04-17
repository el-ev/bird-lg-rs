use common::api::AppResponse;

use crate::store::{
    AppEvent, CommandOutputEvent, LgStateHandle, PingStreamEvent, TracerouteResult,
    TracerouteStreamEvent,
};

pub fn handle_app_response(response: AppResponse, state: &LgStateHandle) {
    if let Some(event) = map_response_to_event(response) {
        tracing::debug!("Dispatching mapped app event");
        state.dispatch(event);
    }
}

pub fn map_response_to_event(response: AppResponse) -> Option<AppEvent> {
    match response {
        AppResponse::Protocols { data } => Some(AppEvent::SetNodes(data)),
        AppResponse::NoChange { last_updated } => Some(AppEvent::UpdateTimestamp(last_updated)),
        AppResponse::ProtocolsDiff { data } => Some(AppEvent::ApplyDiff(data)),
        AppResponse::TracerouteInit { request_id, node } => {
            Some(AppEvent::TracerouteStream(TracerouteStreamEvent::Init {
                request_id,
                node,
            }))
        }
        AppResponse::TracerouteUpdate {
            request_id,
            node,
            hops,
        } => Some(AppEvent::TracerouteStream(TracerouteStreamEvent::Update {
            request_id,
            node,
            result: TracerouteResult::Hops(hops),
        })),
        AppResponse::TracerouteDone { request_id, .. } => {
            Some(AppEvent::TracerouteStream(TracerouteStreamEvent::Done {
                request_id,
            }))
        }
        AppResponse::TracerouteError {
            request_id,
            node,
            error,
        } => Some(AppEvent::TracerouteStream(TracerouteStreamEvent::Error {
            request_id,
            node,
            error,
        })),
        AppResponse::PingInit { request_id, node } => {
            Some(AppEvent::PingStream(PingStreamEvent::Init {
                request_id,
                node,
            }))
        }
        AppResponse::PingUpdate {
            request_id,
            node,
            lines,
        } => Some(AppEvent::PingStream(PingStreamEvent::Update {
            request_id,
            node,
            lines,
        })),
        AppResponse::PingDone { request_id, .. } => {
            Some(AppEvent::PingStream(PingStreamEvent::Done { request_id }))
        }
        AppResponse::PingError {
            request_id,
            node,
            error,
        } => Some(AppEvent::PingStream(PingStreamEvent::Error {
            request_id,
            node,
            error,
        })),
        AppResponse::RouteLookupInit { request_id, .. } => {
            Some(AppEvent::CommandOutputStream(CommandOutputEvent::Init {
                request_id,
            }))
        }
        AppResponse::RouteLookupUpdate {
            request_id, lines, ..
        } => Some(AppEvent::CommandOutputStream(CommandOutputEvent::Update {
            request_id,
            lines,
        })),
        AppResponse::RouteLookupDone { request_id, .. } => {
            Some(AppEvent::CommandOutputStream(CommandOutputEvent::Done {
                request_id,
            }))
        }
        AppResponse::RouteLookupError {
            request_id, error, ..
        } => Some(AppEvent::CommandOutputStream(CommandOutputEvent::Error {
            request_id,
            error,
        })),
        AppResponse::ProtocolDetailsInit { request_id, .. } => {
            Some(AppEvent::CommandOutputStream(CommandOutputEvent::Init {
                request_id,
            }))
        }
        AppResponse::ProtocolDetailsUpdate {
            request_id, lines, ..
        } => Some(AppEvent::CommandOutputStream(CommandOutputEvent::Update {
            request_id,
            lines,
        })),
        AppResponse::ProtocolDetailsDone { request_id, .. } => {
            Some(AppEvent::CommandOutputStream(CommandOutputEvent::Done {
                request_id,
            }))
        }
        AppResponse::ProtocolDetailsError {
            request_id, error, ..
        } => Some(AppEvent::CommandOutputStream(CommandOutputEvent::Error {
            request_id,
            error,
        })),
        AppResponse::WireGuard { data } => Some(AppEvent::SetWireGuard(data)),
        AppResponse::NetworkInfo(info) => Some(AppEvent::SetNetworkInfo(info)),
        AppResponse::Error(e) => {
            tracing::error!("AppResponse Error: {}", e);
            Some(AppEvent::SetError(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use common::api::AppResponse;

    use super::map_response_to_event;
    use crate::store::{AppEvent, CommandOutputEvent, PingStreamEvent, TracerouteStreamEvent};

    #[test]
    fn ping_done_maps_to_single_end_event() {
        let event = map_response_to_event(AppResponse::PingDone {
            request_id: "req-1".to_string(),
            node: "node-a".to_string(),
        });

        assert!(matches!(
            event,
            Some(AppEvent::PingStream(PingStreamEvent::Done { request_id }))
                if request_id == "req-1"
        ));
    }

    #[test]
    fn traceroute_error_maps_to_single_error_event() {
        let event = map_response_to_event(AppResponse::TracerouteError {
            request_id: "req-1".to_string(),
            node: "node-a".to_string(),
            error: "timeout".to_string(),
        });

        assert!(matches!(
            event,
            Some(AppEvent::TracerouteStream(TracerouteStreamEvent::Error {
                request_id,
                node,
                error,
            })) if request_id == "req-1" && node == "node-a" && error == "timeout"
        ));
    }

    #[test]
    fn route_lookup_update_maps_to_single_output_event() {
        let event = map_response_to_event(AppResponse::RouteLookupUpdate {
            request_id: "req-1".to_string(),
            node: "node-a".to_string(),
            lines: vec!["route".to_string()],
        });

        assert!(matches!(
            event,
            Some(AppEvent::CommandOutputStream(CommandOutputEvent::Update {
                request_id,
                lines,
            })) if request_id == "req-1" && lines == vec!["route".to_string()]
        ));
    }
}
