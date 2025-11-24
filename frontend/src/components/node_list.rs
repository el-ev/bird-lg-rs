use chrono::Local;
use yew::prelude::*;

use crate::components::shell::ShellLine;
use crate::config::username;
use crate::models::NodeStatus;

#[derive(Properties, PartialEq)]
pub struct NodeListProps {
    pub nodes: Vec<NodeStatus>,
    pub on_protocol_click: Callback<(String, String)>,
}

#[function_component(NodeList)]
pub fn node_list(props: &NodeListProps) -> Html {
    html! {
        <div>
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
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>{ "Proto" }</th>
                                    <th>{ "Name" }</th>
                                    <th>{ "Table" }</th>
                                    <th>{ "State" }</th>
                                    <th>{ "Since" }</th>
                                    <th>{ "Info" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for node.protocols.iter().map(|p| {
                                    let name_for_click = node_name.clone();
                                    let proto_name = p.name.clone();
                                    let on_row_click = on_protocol_click.clone();
                                    html! {
                                        <tr
                                            class="clickable-row"
                                            onclick={move |_| on_row_click.emit((name_for_click.clone(), proto_name.clone()))}
                                        >
                                            <td>{ &p.proto }</td>
                                            <td>{ &p.name }</td>
                                            <td>{ &p.table }</td>
                                            <td>{ &p.state }</td>
                                            <td>{ &p.since }</td>
                                            <td>{ &p.info }</td>
                                        </tr>
                                    }
                                }) }
                            </tbody>
                        </table>
                    </details>
                }
            }) }
        </div>
    }
}
