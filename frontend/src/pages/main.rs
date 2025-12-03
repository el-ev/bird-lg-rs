use yew::prelude::*;

use crate::{
    components::{
        node_list::NodeList, route_lookup::RouteLookup, traceroute::Traceroute,
        wireguard::WireGuard,
    },
    store::route_info::RouteInfoHandle,
};

#[function_component(MainPage)]
pub fn main_page() -> Html {
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");

    html! {
        <>
            <NodeList/>

            if route_info.wireguard_info.is_some() {
                <WireGuard/>
            }

            <Traceroute/>

            <RouteLookup/>
        </>
    }
}
