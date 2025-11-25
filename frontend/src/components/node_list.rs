use chrono::Local;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::data_table::{DataTable, TableRow};
use crate::components::shell::ShellLine;
use crate::config::{backend_api, username};
use crate::models::NodeStatus;
use crate::services::stream_fetch;
use crate::store::modal::ModalAction;
use crate::store::{Action, AppState};
use crate::utils::filter_protocol_details;

#[derive(Properties, PartialEq)]
pub struct NodeListProps {
    pub nodes: Vec<NodeStatus>,
    pub on_protocol_click: Callback<(String, String)>,
}

#[function_component(NodeList)]
pub fn node_list(props: &NodeListProps) -> Html {
    html! {
        <div>
            <h3>{"Protocols"}</h3>
            { for props.nodes.iter().map(|node| {
                let node_name = node.name.clone();
                let on_protocol_click = props.on_protocol_click.clone();
                html! {
                    <details class="expandable-item" open=true>
                        <summary class="summary-header">
                            <span class="item-title">{ &node.name }</span>
                            <span class="item-meta">
                                {
                                    format!(
                                        "(Updated: {})",
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
                            prompt={format!("{}@{}$ ", username(), node.name)}
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
                                .map(str::to_string)
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
                }
            }) }
        </div>
    }
}

pub fn handle_protocol_click(node: String, proto: String, state: UseReducerHandle<AppState>) {
    state.dispatch(Action::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(format!(
            "{}@{}$ birdc show protocols all {}",
            username(),
            node,
            proto
        )),
    }));

    spawn_local(async move {
        let url = backend_api(&format!("/api/protocols/{}/{}", node, proto));
        let mut aggregated = String::new();
        let result = stream_fetch(url, {
            let state = state.clone();
            move |chunk| {
                aggregated.push_str(&chunk);
                let filtered = filter_protocol_details(&aggregated);
                state.dispatch(Action::Modal(ModalAction::UpdateContent(filtered)));
            }
        })
        .await;

        if let Err(err) = result {
            state.dispatch(Action::Modal(ModalAction::UpdateContent(format!(
                "Failed to load protocol details: {}",
                err
            ))));
        }
    });
}
