use yew::{Html, function_component, html};

use crate::components::{
    node_list::NodeList, route_lookup::RouteLookup, traceroute::Traceroute, wireguard::WireGuard,
};

#[function_component(ProtocolPage)]
pub fn protocol_page() -> Html {
    html! {
        <>
            <NodeList/>

            <WireGuard/>

            <Traceroute/>

            <RouteLookup/>
        </>
    }
}
