use common::models::NetworkInfo;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub node_name: Option<String>,
    pub network_info: Option<NetworkInfo>,
    pub nodes_count: usize,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    html! {
        <h2 class="title title-flex">
            <Link<Route> to={Route::Home} classes="title-link">{"Looking Glass"}</Link<Route>>
            {
                html! {
                    <span class="title-footnote">
                        if let Some(ref info) = props.network_info {
                            { " of " } { &info.name } { " " } { &info.asn } {" on DN42 "}
                        }
                        if let Some(name) = &props.node_name {
                            if props.nodes_count > 0 {
                                { " / " } { name }
                            }
                        }
                    </span>
                }
            }
        </h2>
    }
}
