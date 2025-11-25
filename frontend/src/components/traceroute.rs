use futures::future::join_all;
use serde_json::from_str;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::components::data_table::{DataTable, TableRow};
use crate::components::shell::{ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect};
use crate::models::{NodeStatus, TracerouteHop};
use crate::services::{log_error, stream_fetch};
use crate::store::traceroute::TracerouteAction;
use crate::store::{Action, AppState, NodeTracerouteResult};
use crate::utils::validate_hostname_or_ip;

#[derive(Properties, PartialEq)]
pub struct TracerouteProps {
    pub state: UseReducerHandle<AppState>,
    pub nodes: Vec<NodeStatus>,
}

#[function_component(Traceroute)]
pub fn traceroute_section(props: &TracerouteProps) -> Html {
    let state = props.state.clone();
    let traceroute_state = &state.traceroute;

    let nodes = props.nodes.clone();

    let on_node_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            state.dispatch(Action::Traceroute(TracerouteAction::SetNode(
                target.value(),
            )));
        })
    };

    let on_version_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            state.dispatch(Action::Traceroute(TracerouteAction::SetVersion(
                target.value(),
            )));
        })
    };

    let on_target_change = {
        let state = state.clone();
        Callback::from(move |value: String| {
            state.dispatch(Action::Traceroute(TracerouteAction::SetTarget(value)));
        })
    };

    let on_submit = {
        let state = state.clone();
        let nodes = nodes.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let raw_target = state.traceroute.target.clone();
            let sanitized_target = raw_target.trim().to_string();
            if let Err(err) = validate_hostname_or_ip(&sanitized_target) {
                state.dispatch(Action::Traceroute(TracerouteAction::SetError(err)));
                return;
            }
            state.dispatch(Action::Traceroute(TracerouteAction::ClearError));
            state.dispatch(Action::Traceroute(TracerouteAction::SetLastParams(
                sanitized_target.clone(),
                state.traceroute.version.clone(),
            )));

            state.dispatch(Action::Traceroute(TracerouteAction::Start));

            let validated_target = sanitized_target;
            let nodes = nodes.clone();
            let traceroute_node = state.traceroute.node.clone();
            let traceroute_version = state.traceroute.version.clone();
            let state = state.clone();

            spawn_local(async move {
                let selected_node = traceroute_node;
                let target_nodes = if selected_node.is_empty() {
                    nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>()
                } else {
                    vec![selected_node.clone()]
                };

                let version_value = traceroute_version;
                let target_value = validated_target;

                let futures = target_nodes.into_iter().map(|node_name| {
                    let version_value = version_value.clone();
                    let target_value = target_value.clone();
                    let state = state.clone();

                    async move {
                        let version_param = match version_value.as_str() {
                            "4" => "&version=4",
                            "6" => "&version=6",
                            _ => "",
                        };

                        let url = format!(
                            "{}/api/traceroute?node={}&target={}{}",
                            state.backend_url, node_name, target_value, version_param
                        );

                        state.dispatch(Action::Traceroute(TracerouteAction::InitResult(
                            node_name.clone(),
                        )));

                        let mut buffer = String::new();
                        let mut node_hops: Vec<TracerouteHop> = Vec::new();
                        let stream_result = stream_fetch(url, {
                            let state = state.clone();
                            let node_name = node_name.clone();
                            move |chunk| {
                                buffer.push_str(&chunk);
                                while let Some(idx) = buffer.find('\n') {
                                    let line = buffer[..idx].trim().to_string();
                                    buffer.drain(..=idx);
                                    if line.is_empty() {
                                        continue;
                                    }
                                    if let Ok(hop) = from_str::<TracerouteHop>(&line) {
                                        node_hops.push(hop);
                                        state.dispatch(Action::Traceroute(
                                            TracerouteAction::UpdateResult(
                                                node_name.clone(),
                                                NodeTracerouteResult::Hops(node_hops.clone()),
                                            ),
                                        ));
                                    }
                                }
                            }
                        })
                        .await;

                        if let Err(err) = stream_result {
                            log_error(&format!(
                                "Traceroute stream failed for {}: {}",
                                node_name, err
                            ));
                            let message = format!("Traceroute failed: {}", err);
                            state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                                node_name,
                                NodeTracerouteResult::Error(message),
                            )));
                        }
                    }
                });

                join_all(futures).await;

                state.dispatch(Action::Traceroute(TracerouteAction::End));
            });
        })
    };

    html! {
        <section>
            <h3>{"Traceroute"}</h3>
            <form class="shell-line" onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", state.username)}
                    <ShellSelect
                        class="node-select"
                        value={traceroute_state.node.clone()}
                        on_change={on_node_change}
                    >
                        {
                            if nodes.len() > 1 {
                                html! { <option value="" selected=true>{"(all)"}</option> }
                            } else {
                                html! {}
                            }
                        }
                        { for nodes.iter().map(|n| html! {
                            <option value={n.name.clone()}>{ &n.name }</option>
                        }) }
                    </ShellSelect>
                    {"$ "}
                </ShellPrompt>
                { "traceroute " }
                <ShellSelect
                    value={traceroute_state.version.clone()}
                    on_change={on_version_change}
                >
                    <option value="auto" selected=true>{"  "}</option>
                    <option value="4">{"-4"}</option>
                    <option value="6">{"-6"}</option>
                </ShellSelect>
                <span>{ " " }</span>
                <ShellInput
                    value={traceroute_state.target.clone()}
                    on_change={on_target_change}
                    placeholder="<target>"
                />
                <ShellButton
                    type_="submit"
                    disabled={traceroute_state.loading}
                >
                    { if traceroute_state.loading { "..." } else { "↵" } }
                </ShellButton>
            </form>
            {
                if let Some(err) = &traceroute_state.error {
                    html! { <div class="error-message">{ err }</div> }
                } else {
                    html! {}
                }
            }
            <div>
                { for traceroute_state.results.iter().map(|(node_name, result)| {
                    let version_flag = match traceroute_state.last_version.as_str() {
                        "4" => " -4",
                        "6" => " -6",
                        _ => "",
                    };
                    html! {
                        <details class="expandable-item" open=true>
                            <summary class="summary-header">
                                <h4 class="item-title">{ node_name }</h4>
                            </summary>
                            <ShellLine
                                prompt={format!("{}@{}$ ", state.username, node_name)}
                                command={format!("traceroute{} {}", version_flag, traceroute_state.last_target)}
                                style={"font-size: 0.9em;".to_string()}
                            />
                            {
                                match result {
                                    NodeTracerouteResult::Hops(hops) => html! {
                                        <DataTable
                                            headers={
                                                [
                                                    "Hop",
                                                    "Host",
                                                    "IP",
                                                    "RTTs",
                                                ]
                                                .map(str::to_string)
                                                .to_vec()
                                            }
                                            rows={
                                                hops.iter().map(|hop| {
                                                    TableRow {
                                                        cells: vec![
                                                            html! { hop.hop },
                                                            html! { hop.hostname.clone().unwrap_or_default() },
                                                            html! { hop.address.clone().unwrap_or_default() },
                                                            html! {
                                                                {
                                                                    if let Some(rtts) = &hop.rtts {
                                                                        rtts
                                                                            .iter()
                                                                            .map(|r| format!("{:.2}ms", r))
                                                                            .collect::<Vec<_>>()
                                                                            .join(" / ")
                                                                    } else {
                                                                        "*".to_string()
                                                                    }
                                                                }
                                                            },
                                                        ],
                                                        on_click: None,
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                            }
                                        />
                                    },
                                    NodeTracerouteResult::Error(message) => html! {
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
