use serde_json::from_str;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::config::backend_api;
use crate::models::{NodeStatus, TracerouteHop};
use crate::services::{log_error, stream_fetch};
use crate::utils::validate_hostname_or_ip;

#[derive(Properties, PartialEq)]
pub struct TracerouteSectionProps {
    pub nodes: Vec<NodeStatus>,
}

#[function_component(TracerouteSection)]
pub fn traceroute_section(props: &TracerouteSectionProps) -> Html {
    let traceroute_results = use_state(Vec::<(String, Vec<TracerouteHop>)>::new);
    let traceroute_results_cache = use_mut_ref(Vec::<(String, Vec<TracerouteHop>)>::new);
    let traceroute_target = use_state(String::new);
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

    let on_target_input = {
        let traceroute_target = traceroute_target.clone();
        let traceroute_error = traceroute_error.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            traceroute_target.set(target.value());
            traceroute_error.set(None);
        })
    };

    let on_submit = {
        let traceroute_results = traceroute_results.clone();
        let traceroute_target = traceroute_target.clone();
        let traceroute_node = traceroute_node.clone();
        let traceroute_version = traceroute_version.clone();
        let traceroute_loading = traceroute_loading.clone();
        let traceroute_error = traceroute_error.clone();
        let nodes = nodes_for_form.clone();
        let results_cache = traceroute_results_cache.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let raw_target = (*traceroute_target).clone();
            let sanitized_target = raw_target.trim().to_string();
            if let Err(err) = validate_hostname_or_ip(&sanitized_target) {
                traceroute_error.set(Some(err));
                return;
            }
            traceroute_error.set(None);

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

                for node_name in target_nodes {
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
                        cache.push((node_name.clone(), Vec::new()));
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
                                    if let Some((_, hops)) =
                                        cache.iter_mut().find(|(name, _)| name == &node_for_stream)
                                    {
                                        *hops = node_hops.clone();
                                    } else {
                                        cache.push((node_for_stream.clone(), node_hops.clone()));
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
                        if let Some((_, hops)) =
                            cache.iter_mut().find(|(name, _)| name == &node_name)
                        {
                            *hops = vec![TracerouteHop {
                                hop: 0,
                                address: None,
                                hostname: Some(format!("Error: {}", err)),
                                rtts: None,
                            }];
                        } else {
                            cache.push((
                                node_name.clone(),
                                vec![TracerouteHop {
                                    hop: 0,
                                    address: None,
                                    hostname: Some(format!("Error: {}", err)),
                                    rtts: None,
                                }],
                            ));
                        }
                        let snapshot = cache.clone();
                        drop(cache);
                        stream_state.set(snapshot);
                    }
                }

                traceroute_loading.set(false);
            });
        })
    };

    html! {
        <section class="traceroute-section">
            <h3>{"Traceroute"}</h3>
            <form class="traceroute-form" onsubmit={on_submit}>
                <select value={(*traceroute_node).clone()} onchange={on_node_change}>
                    <option value="" selected=true>{"All Nodes"}</option>
                    { for nodes_for_form.iter().map(|n| {
                        html! { <option value={n.name.clone()}>{ &n.name }</option> }
                    }) }
                </select>
                <select value={(*traceroute_version).clone()} onchange={on_version_change}>
                    <option value="auto" selected=true>{"Auto"}</option>
                    <option value="4">{"IPv4"}</option>
                    <option value="6">{"IPv6"}</option>
                </select>
                <input
                    class="target-input"
                    type="text"
                    placeholder="Target IP/Hostname"
                    value={(*traceroute_target).clone()}
                    oninput={on_target_input}
                />
                <button type="submit" disabled={*traceroute_loading}>
                    { if *traceroute_loading { "Running..." } else { "Run" } }
                </button>
            </form>
            {
                if let Some(err) = &*traceroute_error {
                    html! { <div class="traceroute-error">{ err }</div> }
                } else {
                    html! {}
                }
            }
            <div class="traceroute-results">
                { for traceroute_results.iter().map(|(node_name, hops)| {
                    html! {
                        <div class="traceroute-node">
                            <h4>{ node_name }</h4>
                            <table class="traceroute-table">
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
                        </div>
                    }
                }) }
            </div>
        </section>
    }
}
