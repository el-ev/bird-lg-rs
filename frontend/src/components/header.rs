use common::models::NetworkInfo;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::route_dropdown::RouteDropdown, routes::Route, store::route_info::RouteInfoHandle,
    utils::select_text,
};

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub network_info: Option<NetworkInfo>,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    html! {
        <h2 class="title title-flex">
            <Link<Route> to={Route::Protocols} classes="title-link">{"Looking Glass"}</Link<Route>>
            {
                html! {
                    <span class="title-footnote">
                        if let Some(ref info) = props.network_info {
                            { " of " }
                            <span onclick={select_text}> { &info.name } </span>
                            { " " }
                            <span onclick={select_text}> { &info.asn } </span>
                            { " on " }
                            <a href="https://dn42.dev" style="color: inherit;"> {"DN42"} </a>
                        }
                        <RouteDropdown
                            current_path={route_info.path.clone()}
                            current_node={route_info.node_name.clone()}
                        />
                    </span>
                }
            }
        </h2>
    }
}
