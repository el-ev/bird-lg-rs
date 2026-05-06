use common::{models::NodeProtocol, traceroute::fold_timeouts, utils::validate_target};
use ui_components::shell::{
    ShellButton, ShellForm, ShellInput, ShellLine, ShellPrompt, ShellSelect,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::data_table::{DataTable, TableRow};
use crate::{
    services::api::perform_traceroute,
    store::{LgStateHandle, TracerouteResult, route_info::RouteInfoHandle},
};

#[function_component(Traceroute)]
pub fn traceroute_section() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let selected_node = use_state(String::new);
    let target = use_state(String::new);
    let version = use_state(String::new);
    let error = use_state(|| None::<String>);

    let nodes: Vec<NodeProtocol> = route_info.scoped_protocol_nodes(state.nodes.as_slice());
    let is_single_node = nodes.len() == 1;

    let active_session = state.traceroute.active_session();

    let on_node_change = {
        let selected_node = selected_node.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            selected_node.set(target.value());
        })
    };

    let on_version_change = {
        let version = version.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            version.set(target.value());
        })
    };

    let on_target_change = {
        let target = target.clone();
        let error = error.clone();
        Callback::from(move |value: String| {
            target.set(value);
            error.set(None);
        })
    };

    let on_submit = {
        let error = error.clone();
        let selected_node = selected_node.clone();
        let target = target.clone();
        let version = version.clone();
        let state = state.clone();
        let nodes = nodes.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let target_value = (*target).trim().to_string();
            if let Err(err) = validate_target(&target_value) {
                error.set(Some(err));
                return;
            }

            let selected_node_value = (*selected_node).clone();
            let target_nodes = if selected_node_value.is_empty() {
                nodes
                    .iter()
                    .map(|node| node.name.clone())
                    .collect::<Vec<_>>()
            } else {
                vec![selected_node_value]
            };

            if target_nodes.is_empty() {
                error.set(Some("No nodes available".to_string()));
                return;
            }

            error.set(None);
            let version_value = (*version).clone();
            let state_async = state.clone();

            spawn_local(async move {
                if let Err(fetch_error) =
                    perform_traceroute(&state_async, target_nodes, target_value, version_value)
                        .await
                {
                    tracing::error!("Traceroute request failed: {}", fetch_error);
                }
            });
        })
    };

    html! {
        <section>
            <h3>{"Traceroute"}</h3>
            <ShellForm onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", state.username)}
                    {
                        if is_single_node {
                            html! { { &nodes[0].name } }
                        } else {
                            html! {
                                <ShellSelect
                                    class="node-select"
                                    value={(*selected_node).clone()}
                                    on_change={on_node_change}
                                >
                                    {
                                        if nodes.len() > 1 {
                                            html! { <option value="" selected=true>{"(all)"}</option> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    { for nodes.iter().map(|node| html! {
                                        <option value={node.name.clone()}>{ &node.name }</option>
                                    }) }
                                </ShellSelect>
                            }
                        }
                    }
                    {"$ "}
                </ShellPrompt>
                { "traceroute " }
                <ShellSelect value={(*version).clone()} on_change={on_version_change}>
                    <option value="" selected=true>{"  "}</option>
                    <option value="4">{"-4"}</option>
                    <option value="6">{"-6"}</option>
                </ShellSelect>
                <span>{ " " }</span>
                <ShellInput
                    value={(*target).clone()}
                    on_change={on_target_change}
                    placeholder="<target>"
                />
                <ShellButton type_="submit" class="shell-button--submit">
                    { if state.traceroute.is_loading() { "..." } else { "↵" } }
                </ShellButton>
            </ShellForm>
            {
                if let Some(err) = &*error {
                    html! { <div class="error-message">{ err }</div> }
                } else {
                    html! {}
                }
            }
            <div>
                { for nodes.iter().filter_map(|node| {
                    active_session.and_then(|session| {
                        session
                            .results
                            .iter()
                            .find(|(node_name, _)| node_name == &node.name)
                            .map(|result| (&node.name, result))
                    })
                }).map(|(node_name, (_, result))| {
                    let version_flag = active_session.map(|session| session.version.clone()).unwrap_or_default();
                    let version_flag = match version_flag.as_str() {
                        "4" => " -4",
                        "6" => " -6",
                        _ => "",
                    };
                    let target_value = active_session.map(|session| session.target.clone()).unwrap_or_default();

                    html! {
                        <details class="expandable-item" open=true>
                            <summary class="summary-header">
                                <h4 class="item-title">{ node_name }</h4>
                            </summary>
                            <ShellLine
                                prompt={format!("{}@{}$ ", state.username, node_name)}
                                command={format!("traceroute{} {}", version_flag, target_value)}
                                style={"font-size: 0.9em;".to_string()}
                            />
                            {
                                match result {
                                    TracerouteResult::Hops(hops) => html! {
                                        <DataTable
                                            headers={
                                                ["Hop", "Host", "IP", "RTTs"]
                                                    .map(AttrValue::from)
                                                    .to_vec()
                                            }
                                            rows={
                                                fold_timeouts(hops).iter().map(|hop| {
                                                    TableRow {
                                                        cells: vec![
                                                            html! { hop.hop.to_string() },
                                                            html! { hop.hostname.clone().unwrap_or_default() },
                                                            html! { hop.address.clone().unwrap_or_default() },
                                                            html! {
                                                                {
                                                                    hop.rtts
                                                                        .as_ref()
                                                                        .map(|rtts| {
                                                                            rtts.iter()
                                                                                .map(|rtt| format!("{:.2}ms", rtt))
                                                                                .collect::<Vec<_>>()
                                                                                .join(" / ")
                                                                        })
                                                                        .unwrap_or_else(|| "*".to_string())
                                                                }
                                                            },
                                                        ],
                                                        on_click: None,
                                                    }
                                                }).collect::<Vec<_>>()
                                            }
                                        />
                                    },
                                    TracerouteResult::Error(message) => html! {
                                        <pre class="status-message--error">{ message }</pre>
                                    },
                                }
                            }
                        </details>
                    }
                }) }
            </div>
        </section>
    }
}
