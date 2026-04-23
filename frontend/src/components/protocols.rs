use chrono::Local;
use ui_components::shell::ShellLine;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use super::data_table::{DataTable, TableRow};
use crate::{
    services::api::get_protocol_details,
    store::{LgStateHandle, route_info::RouteInfoHandle},
};

#[function_component(Protocols)]
pub fn protocols() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let node_route = route_info.node_name.is_some();
    let on_protocol_click = {
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
                                            if node.protocols.is_empty() && node.error.is_none() {
                                                html! { <p class="status-message">{"No protocol sessions found"}</p> }
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
