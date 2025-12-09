use chrono::Local;
use yew::prelude::*;

use super::{
    data_table::{DataTable, TableRow},
    shell::ShellLine,
};
use crate::{
    services::api::get_protocol_details,
    store::{LgStateHandle, route_info::RouteInfoHandle},
};

#[function_component(Protocols)]
pub fn protocols() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let on_protocol_click = {
        let state = state.clone();
        Callback::from(move |(node, proto): (String, String)| {
            get_protocol_details(&state, node, proto);
        })
    };
    let nodes = if let Some(node) = &route_info.node_info {
        std::slice::from_ref(node)
    } else {
        state.nodes.as_slice()
    };
    html! {
        <div>
            <h3>{"Protocols"}</h3>
            { for nodes.iter().map(|node| {
                let node_name = node.name.clone();
                let on_protocol_click = on_protocol_click.clone();
                html! {
                    <>
                        <details class="expandable-item" open=true>
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
                        </details>
                    </>
                }
            }) }
        </div>
    }
}
