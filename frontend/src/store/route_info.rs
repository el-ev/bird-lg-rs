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

impl RouteInfo {
    pub fn scoped_protocol_nodes(&self, nodes: &[NodeProtocol]) -> Vec<NodeProtocol> {
        match (&self.node_name, &self.node_info) {
            (_, Some(node)) => vec![node.clone()],
            (Some(_), None) => Vec::new(),
            (None, None) => nodes.to_vec(),
        }
    }

    pub fn scoped_wireguard_nodes(&self, nodes: &[NodeWireGuard]) -> Vec<NodeWireGuard> {
        match (&self.node_name, &self.wireguard_info) {
            (_, Some(node)) => vec![node.clone()],
            (Some(_), None) => Vec::new(),
            (None, None) => nodes.to_vec(),
        }
    }
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
            Route::NodeProtocol { node, .. } => Some(node.clone()),
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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::models::{NodeProtocol, NodeWireGuard};

    use super::RouteInfo;

    fn protocol_node(name: &str) -> NodeProtocol {
        NodeProtocol {
            name: name.to_string(),
            protocols: Vec::new(),
            last_updated: Utc::now(),
            error: None,
        }
    }

    fn wireguard_node(name: &str) -> NodeWireGuard {
        NodeWireGuard {
            name: name.to_string(),
            peers: Vec::new(),
            last_updated: Utc::now(),
            error: None,
        }
    }

    #[test]
    fn scoped_protocol_nodes_do_not_fall_back_to_global_data_for_missing_node_routes() {
        let nodes = vec![protocol_node("edge-a"), protocol_node("edge-b")];
        let route_info = RouteInfo {
            path: "/node/edge-c/".to_string(),
            node_name: Some("edge-c".to_string()),
            node_info: None,
            wireguard_info: None,
        };

        assert!(route_info.scoped_protocol_nodes(&nodes).is_empty());
    }

    #[test]
    fn scoped_protocol_nodes_return_selected_node_when_available() {
        let route_info = RouteInfo {
            path: "/node/edge-a/".to_string(),
            node_name: Some("edge-a".to_string()),
            node_info: Some(protocol_node("edge-a")),
            wireguard_info: None,
        };

        assert_eq!(
            route_info
                .scoped_protocol_nodes(&[protocol_node("edge-a"), protocol_node("edge-b")])
                .into_iter()
                .map(|node| node.name)
                .collect::<Vec<_>>(),
            vec!["edge-a".to_string()]
        );
    }

    #[test]
    fn scoped_wireguard_nodes_do_not_fall_back_to_global_data_for_missing_node_routes() {
        let nodes = vec![wireguard_node("edge-a"), wireguard_node("edge-b")];
        let route_info = RouteInfo {
            path: "/node/edge-c/".to_string(),
            node_name: Some("edge-c".to_string()),
            node_info: None,
            wireguard_info: None,
        };

        assert!(route_info.scoped_wireguard_nodes(&nodes).is_empty());
    }
}
