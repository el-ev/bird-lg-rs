use chrono::Local;
use yew::prelude::*;

use crate::models::NodeStatus;

#[derive(Properties, PartialEq)]
pub struct NodeListProps {
    pub nodes: Vec<NodeStatus>,
    pub on_protocol_click: Callback<(String, String)>,
}

#[function_component(NodeList)]
pub fn node_list(props: &NodeListProps) -> Html {
    html! {
        <div class="nodes">
            { for props.nodes.iter().map(|node| {
                let node_name = node.name.clone();
                let on_protocol_click = props.on_protocol_click.clone();
                html! {
                    <details class="node" open=true>
                        <summary class="node-summary">
                            <span class="node-name">{ &node.name }</span>
                            <span class="node-updated">
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
                                        html! { <span class="error-pill">{ "ERR" }</span> }
                                    } else {
                                        html! {}
                                    }
                                }
                            </span>
                        </summary>
                        <table class="node-table">
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
                                            class="node-row"
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
