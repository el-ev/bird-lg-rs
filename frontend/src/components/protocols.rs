use chrono::Local;
use ui_components::shell::ShellLine;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

use super::data_table::{DataTable, TableRow};
use crate::{
    services::api::get_protocol_details,
    store::{LgStateHandle, route_info::RouteInfoHandle},
    utils::{clear_hash_route, resolve_hash_protocol, set_hash_route},
};

#[function_component(Protocols)]
pub fn protocols() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let node_route = route_info.node_name.is_some();

    let open_protocol = {
        let state = state.clone();
        Callback::from(move |(node, proto): (String, String)| {
            let state = state.clone();
            spawn_local(async move {
                if let Err(error) = get_protocol_details(&state, node, proto).await {
                    tracing::error!("Failed to load protocol details: {}", error);
                }
            });
        })
    };

    let on_protocol_click = {
        let open_protocol = open_protocol.clone();
        Callback::from(move |(node, proto): (String, String)| {
            let hash = if node_route {
                proto.clone()
            } else {
                format!("{node}/{proto}")
            };
            set_hash_route(&hash);
            open_protocol.emit((node, proto));
        })
    };

    {
        let state = state.clone();
        let route_info = route_info.clone();
        let open_protocol = open_protocol.clone();
        let handled = use_mut_ref(|| false);
        use_effect_with(state.data_ready, move |&ready| {
            if ready && !*handled.borrow() {
                *handled.borrow_mut() = true;
                if let Some((node, proto)) =
                    resolve_hash_protocol(route_info.node_name.as_deref(), &state.nodes)
                {
                    open_protocol.emit((node, proto));
                } else if crate::utils::get_hash_route().is_some() {
                    clear_hash_route();
                }
            }
        });
    }

    {
        let state_ref = use_mut_ref(|| state.clone());
        *state_ref.borrow_mut() = state.clone();
        let route_info_ref = use_mut_ref(|| route_info.clone());
        *route_info_ref.borrow_mut() = route_info.clone();
        let open_protocol = open_protocol.clone();
        let close = {
            let state = state.clone();
            Callback::from(move |_: ()| {
                state.dispatch(crate::store::AppEvent::CloseActiveCommandOutput);
            })
        };
        use_effect(move || {
            let listener = Closure::<dyn Fn()>::wrap(Box::new(move || {
                let state = state_ref.borrow();
                let route_info = route_info_ref.borrow();
                match resolve_hash_protocol(route_info.node_name.as_deref(), &state.nodes) {
                    Some((node, proto)) => open_protocol.emit((node, proto)),
                    None => {
                        close.emit(());
                        if crate::utils::get_hash_route().is_some() {
                            clear_hash_route();
                        }
                    }
                }
            }));
            let win = window().unwrap();
            let _ =
                win.add_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref());
            move || {
                let _ = win.remove_event_listener_with_callback(
                    "popstate",
                    listener.as_ref().unchecked_ref(),
                );
                drop(listener);
            }
        });
    }
    let nodes = route_info.scoped_protocol_nodes(state.nodes.as_slice());
    html! {
        <div>
            <h3>{"Protocols"}</h3>
            {
                if node_route && nodes.is_empty() {
                    html! { <p class="status-message">{"No protocol data loaded for this node"}</p> }
                } else {
                    html! {
                        for nodes.iter().map(|node| {
                            let node_name = node.name.clone();
                            let on_protocol_click = on_protocol_click.clone();
                            html! {
                                <>
                                    <details key={node.name.clone()} class="expandable-item" open=true>
                                        <summary class="summary-header">
                                            <span class="item-title">{ &node.name }</span>
                                            <span class="item-meta">
                                                {
                                                    format!(
                                                        "({}: {})",
                                                        if node.error.is_some() {
                                                            "Last active"
                                                        } else {
                                                            "Updated"
                                                        },
                                                        node
                                                            .last_updated
                                                            .with_timezone(&Local)
                                                            .format("%Y-%m-%d %H:%M:%S")
                                                    )
                                                }
                                                {
                                                    if node.error.is_some() {
                                                        html! { <span class="status-pill">{ "ERR" }</span> }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </span>
                                        </summary>
                                        {
                                            if let Some(error) = &node.error {
                                                html! { <pre class="status-message--error">{ error }</pre> }
                                            } else {
                                                html! {}
                                            }
                                        }
                                        {
                                            if node.protocols.is_empty() {
                                                if node.error.is_none() {
                                                    html! { <p class="status-message">{"No protocol sessions found"}</p> }
                                                } else {
                                                    html! {}
                                                }
                                            } else {
                                                html! {
                                                    <>
                                                        <ShellLine
                                                            prompt={format!("{}@{}$ ", state.username, node.name)}
                                                            command={"birdc show protocols".to_string()}
                                                            style={"font-size: 0.9em;".to_string()}
                                                        />
                                                        <DataTable
                                                            headers={
                                                                [
                                                                    "Proto",
                                                                    "Name",
                                                                    "Table",
                                                                    "State",
                                                                    "Since",
                                                                    "Info",
                                                                ]
                                                                .map(AttrValue::from)
                                                                .to_vec()
                                                            }
                                                            rows={
                                                                node.protocols.iter().map(|p| {
                                                                    let name_for_click = node_name.clone();
                                                                    let proto_name = p.name.clone();
                                                                    let on_row_click = on_protocol_click.clone();
                                                                    TableRow {
                                                                        cells: vec![
                                                                            html! { &p.proto },
                                                                            html! { &p.name },
                                                                            html! { &p.table },
                                                                            html! { &p.state },
                                                                            html! { &p.since },
                                                                            html! { &p.info },
                                                                        ],
                                                                        on_click: Some(Callback::from(move |_| {
                                                                            on_row_click.emit((
                                                                                name_for_click.clone(),
                                                                                proto_name.clone(),
                                                                            ));
                                                                        })),
                                                                    }
                                                                })
                                                                .collect::<Vec<_>>()
                                                            }
                                                        />
                                                    </>
                                                }
                                            }
                                        }
                                    </details>
                                </>
                            }
                        })
                    }
                }
            }
        </div>
    }
}
