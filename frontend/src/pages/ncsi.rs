use yew::prelude::*;

use crate::components::{route_lookup::RouteLookup, traceroute::Traceroute};

#[function_component(NcsiPage)]
pub fn ncsi_page() -> Html {
    html! {
    <>
        <h3>{"Network Connectivity Status Indicator"}</h3>

        <Traceroute/>

        <RouteLookup/>
    </>
    }
}
