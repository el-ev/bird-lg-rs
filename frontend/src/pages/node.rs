use yew::prelude::*;

use crate::components::{
    protocols::Protocols, route_lookup::RouteLookup, traceroute::Traceroute, wireguard::WireGuard,
};

#[function_component(NodePage)]
pub fn node_page() -> Html {
    html! {
        <>
            <Protocols/>

            <WireGuard/>

            <Traceroute/>

            <RouteLookup/>
        </>
    }
}
