use std::rc::Rc;

use common::models::{NodeProtocol, NodeWireGuard};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{routes::Route, store::LgStateHandle};

#[derive(Clone, Debug, PartialEq)]
pub struct RouteInfo {
    pub path: String,
    pub node_name: Option<String>,
    pub wireguard_info: Option<NodeWireGuard>,
    pub node_info: Option<NodeProtocol>,
}

impl Default for RouteInfo {
    fn default() -> Self {
        Self {
            path: String::from("/"),
            node_name: None,
            wireguard_info: None,
            node_info: None,
        }
    }
}

pub type RouteInfoHandle = Rc<RouteInfo>;

#[derive(Properties, PartialEq)]
pub struct RouteInfoProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(RouteInfoProvider)]
pub fn route_info_provider(props: &RouteInfoProviderProps) -> Html {
    let route = use_route::<Route>().unwrap_or(Route::Protocols);
    let app_state = use_context::<LgStateHandle>().expect("no app state found");

    let route_info = use_memo((route.clone(), app_state.clone()), |(route, app_state)| {
        let path = route.to_path();
        let node_name = match route {
            Route::Node { name } => Some(name.clone()),
            _ => None,
        };

        let node_info = node_name
            .as_ref()
            .and_then(|name| app_state.nodes.iter().find(|n| &n.name == name).cloned());

        let wireguard_info = node_name.as_ref().and_then(|name| {
            app_state
                .wireguard
                .iter()
                .find(|wg| &wg.name == name)
                .cloned()
        });

        Rc::new(RouteInfo {
            path,
            node_name,
            node_info,
            wireguard_info,
        })
    });

    html! {
        <ContextProvider<RouteInfoHandle> context={(*route_info).clone()}>
            { for props.children.iter() }
        </ContextProvider<RouteInfoHandle>>
    }
}
