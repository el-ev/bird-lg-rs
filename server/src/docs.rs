use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "bird-lg-rs server API",
        version = "0.1.0",
        description = "HTTP API for the bird-lg-rs backend.\n\nStreamed diagnostic endpoints use Server-Sent Events. Each event's data field contains a JSON-serialized AppResponse payload."
    ),
    paths(
        crate::handlers::status::get_all_protocols,
        crate::handlers::status::get_node_protocols,
        crate::handlers::protocol::get_protocol_details,
        crate::handlers::traceroute::proxy_traceroute,
        crate::handlers::ping::proxy_ping,
        crate::handlers::route::get_route,
        crate::handlers::info::get_network_info,
        crate::handlers::info::get_network_info_with_port,
        crate::handlers::info::get_node_peering,
        crate::handlers::wireguard::get_wireguard_snapshot
    ),
    components(schemas(
        common::api::AppResponse,
        common::models::Protocol,
        common::models::PeeringInfo,
        common::models::NodeProtocol,
        common::models::NetworkInfo,
        common::models::WireGuardPeer,
        common::models::NodeWireGuard,
        common::models::DiffOp,
        common::models::NodeStatusDiff,
        common::traceroute::HopRange,
        common::traceroute::TracerouteHop
    )),
    tags(
        (name = "protocols", description = "Protocol snapshots and protocol detail lookups"),
        (name = "network", description = "Network metadata, peering data, and WireGuard status"),
        (name = "tools", description = "Diagnostic endpoints streamed over Server-Sent Events")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use super::ApiDoc;

    #[test]
    fn documents_public_http_routes() {
        let openapi = ApiDoc::openapi();

        assert!(openapi.paths.paths.contains_key("/api/protocols"));
        assert!(openapi.paths.paths.contains_key("/api/info"));
        assert!(openapi.paths.paths.contains_key("/api/wireguard"));
        assert!(openapi.paths.paths.contains_key("/api/ping/{node_name}"));
    }
}
