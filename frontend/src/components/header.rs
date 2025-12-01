use common::models::NetworkInfo;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{routes::Route, store::route_info::RouteInfoHandle};

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub network_info: Option<NetworkInfo>,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    html! {
        <h2 class="title title-flex">
            <Link<Route> to={Route::Home} classes="title-link">{"Looking Glass"}</Link<Route>>
            {
                html! {
                    <span class="title-footnote">
                        if let Some(ref info) = props.network_info {
                            { " of " } { &info.name } { " " } { &info.asn } {" on DN42 "}
                        }
                        { &route_info.path }
                    </span>
                }
            }
        </h2>
    }
}
