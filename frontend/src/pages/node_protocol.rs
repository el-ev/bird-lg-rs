use std::{cell::Cell, rc::Rc};

use ui_components::shell::ShellLine;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    pages::NotFoundPage,
    routes::Route,
    services::api::fetch_protocol_details_content,
    store::{LgStateHandle, route_info::RouteInfoHandle},
    utils::sleep_ms,
};

const REFRESH_INTERVAL_MS: i32 = 10_000;

#[function_component(NodeProtocolPage)]
pub fn node_protocol_page() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let route = use_route::<Route>();

    let (node_name, protocol_name) = match route {
        Some(Route::NodeProtocol { node, protocol }) => (node, protocol),
        _ => return html! { <NotFoundPage/> },
    };

    let content = use_state(|| None::<String>);

    let protocol_exists = state.data_ready
        && route_info
            .node_info
            .as_ref()
            .is_some_and(|node| node.protocols.iter().any(|p| p.name == protocol_name));

    {
        let content = content.clone();
        let backend_url = state.backend_url.clone();
        let node = node_name.clone();
        let proto = protocol_name.clone();
        use_effect_with(
            (node.clone(), proto.clone(), protocol_exists),
            move |&(ref node, ref proto, exists)| {
                let active = Rc::new(Cell::new(true));
                let cleanup = active.clone();

                if exists {
                    let node = node.clone();
                    let proto = proto.clone();
                    spawn_local(async move {
                        loop {
                            match fetch_protocol_details_content(&backend_url, &node, &proto).await
                            {
                                Ok(text) => content.set(Some(text)),
                                Err(err) => content.set(Some(format!("Error: {err}\n"))),
                            }
                            if !active.get() {
                                break;
                            }
                            sleep_ms(REFRESH_INTERVAL_MS).await;
                            if !active.get() {
                                break;
                            }
                        }
                    });
                }

                move || cleanup.set(false)
            },
        );
    }

    if !state.data_ready {
        return html! { <p class="status-message">{"Loading..."}</p> };
    }

    if !protocol_exists {
        return html! { <NotFoundPage/> };
    }

    html! {
        <div class="protocol-detail">
            <div class="protocol-detail-header">
                <h3 class="protocol-detail-title">{ &protocol_name }</h3>
                <span class="protocol-detail-subtitle">
                    {"on "}
                    <Link<Route> to={Route::Node { name: node_name.clone() }}>
                        { &node_name }
                    </Link<Route>>
                </span>
            </div>

            <div class="protocol-detail-body">
                <div class="protocol-detail-command">
                    <ShellLine
                        prompt={format!("{}@{}$ ", state.username, node_name)}
                        command={format!("birdc show protocols all {}", protocol_name)}
                    />
                </div>
                <pre class="protocol-detail-output">
                    { content.as_deref().unwrap_or("Loading...") }
                </pre>
            </div>
        </div>
    }
}
