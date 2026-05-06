use yew::prelude::*;

use crate::{
    components::{
        protocols::Protocols, route_lookup::RouteLookup, traceroute::Traceroute,
        wireguard::WireGuard,
    },
    pages::NotFoundPage,
    store::{LgStateHandle, route_info::RouteInfoHandle},
};

#[function_component(NodePage)]
pub fn node_page() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");

    if state.data_ready {
        if let Some(name) = &route_info.node_name {
            if !state.nodes.iter().any(|n| &n.name == name) {
                return html! { <NotFoundPage/> };
            }
        }
    }

    html! {
        <>
            <Protocols/>

            <WireGuard/>

            <Traceroute/>

            <RouteLookup/>
        </>
    }
}
