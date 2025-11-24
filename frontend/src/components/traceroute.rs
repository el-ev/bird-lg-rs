use futures::future::join_all;
use serde_json::from_str;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::components::shell::{ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect};
use crate::config::{backend_api, username};
use crate::models::{NodeStatus, TracerouteHop};
use crate::services::{log_error, stream_fetch};
use crate::utils::validate_hostname_or_ip;

#[derive(Properties, PartialEq)]
pub struct TracerouteSectionProps {
    pub nodes: Vec<NodeStatus>,
}

#[derive(Clone, PartialEq)]
enum NodeTracerouteResult {
    Hops(Vec<TracerouteHop>),
    Error(String),
}

#[function_component(TracerouteSection)]
pub fn traceroute_section(props: &TracerouteSectionProps) -> Html {
    let traceroute_results = use_state(Vec::<(String, NodeTracerouteResult)>::new);
    let traceroute_results_cache = use_mut_ref(Vec::<(String, NodeTracerouteResult)>::new);
    let traceroute_target = use_state(String::new);
    let last_traceroute_target = use_state(String::new);
    let last_traceroute_version = use_state(String::new);
    let traceroute_node = use_state(String::new);
    let traceroute_version = use_state(|| "auto".to_string());
    let traceroute_loading = use_state(|| false);
    let traceroute_error = use_state(|| None::<String>);

    let nodes_for_form = props.nodes.clone();

    let on_node_change = {
        let traceroute_node = traceroute_node.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            traceroute_node.set(target.value());
        })
    };

    let on_version_change = {
        let traceroute_version = traceroute_version.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            traceroute_version.set(target.value());
        })
    };

    let on_target_change = {
        let traceroute_target = traceroute_target.clone();
        let traceroute_error = traceroute_error.clone();
        Callback::from(move |value: String| {
            traceroute_target.set(value);
            traceroute_error.set(None);
        })
    };

    let on_submit = {
        let traceroute_results = traceroute_results.clone();
        let traceroute_target = traceroute_target.clone();
        let last_traceroute_target = last_traceroute_target.clone();
        let last_traceroute_version = last_traceroute_version.clone();
        let traceroute_node = traceroute_node.clone();
        let traceroute_version = traceroute_version.clone();
        let traceroute_loading = traceroute_loading.clone();
        let traceroute_error_state = traceroute_error.clone();
        let nodes = nodes_for_form.clone();
        let results_cache = traceroute_results_cache.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let raw_target = (*traceroute_target).clone();
            let sanitized_target = raw_target.trim().to_string();
            if let Err(err) = validate_hostname_or_ip(&sanitized_target) {
                traceroute_error_state.set(Some(err));
                return;
            }
            traceroute_error_state.set(None);
            last_traceroute_target.set(sanitized_target.clone());
            last_traceroute_version.set((*traceroute_version).clone());

            traceroute_loading.set(true);
            {
                let mut cache = results_cache.borrow_mut();
                cache.clear();
            }
            traceroute_results.set(Vec::new());

            let validated_target = sanitized_target;
            let nodes = nodes.clone();
            let traceroute_results = traceroute_results.clone();
            let traceroute_node = traceroute_node.clone();
            let traceroute_version = traceroute_version.clone();
            let traceroute_loading = traceroute_loading.clone();
            let results_cache = results_cache.clone();

            spawn_local(async move {
                let selected_node = (*traceroute_node).clone();
                let target_nodes = if selected_node.is_empty() {
                    nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>()
                } else {
                    vec![selected_node.clone()]
                };

                let version_value = (*traceroute_version).clone();
                let target_value = validated_target;

                let futures = target_nodes.into_iter().map(|node_name| {
                    let version_value = version_value.clone();
                    let target_value = target_value.clone();
                    let results_cache = results_cache.clone();
                    let traceroute_results = traceroute_results.clone();

                    async move {
                        let version_param = match version_value.as_str() {
                            "4" => "&version=4",
                            "6" => "&version=6",
                            _ => "",
                        };

                        let url = backend_api(&format!(
                            "/api/traceroute?node={}&target={}{}",
                            node_name, target_value, version_param
                        ));

                        {
                            let mut cache = results_cache.borrow_mut();
                            cache.retain(|(name, _)| name != &node_name);
                            cache.push((node_name.clone(), NodeTracerouteResult::Hops(Vec::new())));
                            traceroute_results.set(cache.clone());
                        }

                        let stream_state = traceroute_results.clone();
                        let cache_handle = results_cache.clone();
                        let mut buffer = String::new();
                        let mut node_hops: Vec<TracerouteHop> = Vec::new();
                        let stream_result = stream_fetch(url, {
                            let stream_state = stream_state.clone();
                            let cache_handle = cache_handle.clone();
                            let node_for_stream = node_name.clone();
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
                                        let mut cache = cache_handle.borrow_mut();
                                        if let Some((_, result)) = cache
                                            .iter_mut()
                                            .find(|(name, _)| name == &node_for_stream)
                                        {
                                            *result = NodeTracerouteResult::Hops(node_hops.clone());
                                        } else {
                                            cache.push((
                                                node_for_stream.clone(),
                                                NodeTracerouteResult::Hops(node_hops.clone()),
                                            ));
                                        }
                                        let snapshot = cache.clone();
                                        drop(cache);
                                        stream_state.set(snapshot);
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
                            let mut cache = results_cache.borrow_mut();
                            let message = format!("Traceroute failed: {}", err);
                            if let Some((_, result)) =
                                cache.iter_mut().find(|(name, _)| name == &node_name)
                            {
                                *result = NodeTracerouteResult::Error(message.clone());
                            } else {
                                cache.push((
                                    node_name.clone(),
                                    NodeTracerouteResult::Error(message.clone()),
                                ));
                            }
                            let snapshot = cache.clone();
                            drop(cache);
                            stream_state.set(snapshot);
                        }
                    }
                });

                join_all(futures).await;

                traceroute_loading.set(false);
            });
        })
    };

    html! {
        <section>
            <h3>{"Traceroute"}</h3>
            <form class="shell-line" onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", username())}
                    <ShellSelect class="node-select" value={(*traceroute_node).clone()} on_change={on_node_change}>
                        <option value="" selected=true>{"(all)"}</option>
                        { for nodes_for_form.iter().map(|n| {
                            html! { <option value={n.name.clone()}>{ &n.name }</option> }
                        }) }
                    </ShellSelect>
                    {"$ "}
                </ShellPrompt>
                { "traceroute " }
                <ShellSelect value={(*traceroute_version).clone()} on_change={on_version_change}>
                    <option value="auto" selected=true>{"  "}</option>
                    <option value="4">{"-4"}</option>
                    <option value="6">{"-6"}</option>
                </ShellSelect>
                <span>{ " " }</span>
                <ShellInput
                    value={(*traceroute_target).clone()}
                    on_change={on_target_change}
                    placeholder="<target>"
                />
                <ShellButton type_="submit" disabled={*traceroute_loading}>
                    { if *traceroute_loading { "..." } else { "↵" } }
                </ShellButton>
            </form>
            {
                if let Some(err) = &*traceroute_error {
                    html! { <div class="error-message">{ err }</div> }
                } else {
                    html! {}
                }
            }
            <div>
                { for traceroute_results.iter().map(|(node_name, result)| {
                    let version_flag = match last_traceroute_version.as_str() {
                        "4" => " -4",
                        "6" => " -6",
                        _ => "",
                    };
                    html! {
                        <details class="expandable-item" open=true>
                            <summary class="summary-header"><h4 class="item-title">{ node_name }</h4></summary>
                            <ShellLine
                                prompt={format!("{}@{}$ ", username(), node_name)}
                                command={format!("traceroute{} {}", version_flag, *last_traceroute_target)}
                                style={"font-size: 0.9em;".to_string()}
                            />
                            {
                                match result {
                                    NodeTracerouteResult::Hops(hops) => html! {
                                        <table class="data-table">
                                            <thead>
                                                <tr>
                                                    <th>{"Hop"}</th>
                                                    <th>{"Host"}</th>
                                                    <th>{"IP"}</th>
                                                    <th>{"RTTs"}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for hops.iter().map(|hop| {
                                                    html! {
                                                        <tr>
                                                            <td>{ hop.hop }</td>
                                                            <td>{ hop.hostname.clone().unwrap_or_default() }</td>
                                                            <td>{ hop.address.clone().unwrap_or_default() }</td>
                                                            <td>
                                                                {
                                                                    if let Some(rtts) = &hop.rtts {
                                                                        rtts.iter().map(|r| format!("{:.2}ms", r)).collect::<Vec<_>>().join(" / ")
                                                                    } else {
                                                                        "*".to_string()
                                                                    }
                                                                }
                                                            </td>
                                                        </tr>
                                                    }
                                                }) }
                                            </tbody>
                                        </table>
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
