use std::collections::HashMap;

use chrono::{DateTime, Utc};

use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::{
    content_modal::ContentModal, node_list::NodeList, route_lookup::RouteLookup,
    status_banner::StatusBanner, traceroute::TracerouteSection,
};
use crate::config::{backend_api, load_config, username};
use crate::models::NodeStatus;
use crate::services::{log_error, sleep_ms, stream_fetch};
use crate::utils::filter_protocol_details;

#[function_component(App)]
pub fn app() -> Html {
    let nodes = use_state(Vec::<NodeStatus>::new);
    let modal_active = use_state(|| false);
    let modal_content = use_state(String::new);
    let modal_command = use_state(|| None::<String>);
    let fetch_error = use_state(|| None::<String>);
    let data_ready = use_state(|| false);
    let config_ready = use_state(|| false);

    {
        let config_ready = config_ready.clone();
        let fetch_error = fetch_error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match load_config().await {
                    Ok(_) => config_ready.set(true),
                    Err(err) => {
                        let message = format!("Configuration load failed: {}", err);
                        fetch_error.set(Some(message.clone()));
                        log_error(&message);
                    }
                }
            });

            || ()
        });
    }

    {
        let nodes = nodes.clone();
        let fetch_error = fetch_error.clone();
        let data_ready = data_ready.clone();
        use_effect_with(*config_ready, move |ready| {
            if *ready {
                let nodes_handle = nodes.clone();
                let fetch_error_handle = fetch_error.clone();
                let data_ready_handle = data_ready.clone();

                spawn_local(async move {
                    let mut last_known_updates = HashMap::new();
                    loop {
                        fetch_status(
                            &nodes_handle,
                            &fetch_error_handle,
                            &data_ready_handle,
                            &mut last_known_updates,
                        )
                        .await;
                        sleep_ms(5000).await;
                    }
                });
            }

            || ()
        });
    }

    let on_protocol_click = {
        let modal_active = modal_active.clone();
        let protocol_details = modal_content.clone();
        let modal_command = modal_command.clone();
        Callback::from(move |(node, proto): (String, String)| {
            handle_protocol_click(
                node,
                proto,
                modal_active.clone(),
                protocol_details.clone(),
                modal_command.clone(),
            );
        })
    };

    let on_route_lookup = {
        let modal_active = modal_active.clone();
        let protocol_details = modal_content.clone();
        let modal_command = modal_command.clone();
        Callback::from(move |(node, target, all): (String, String, bool)| {
            handle_route_lookup(
                node,
                target,
                all,
                modal_active.clone(),
                protocol_details.clone(),
                modal_command.clone(),
            );
        })
    };

    let waiting_for_data = nodes.is_empty() && (!*data_ready || !*config_ready);

    html! {
        <main class="hero">
            <div class="container">
                <h2 class="title title-flex">{"Looking Glass"}</h2>

                <StatusBanner
                    fetch_error={(*fetch_error).clone()}
                    waiting_for_data={waiting_for_data}
                />

                <NodeList
                    nodes={(*nodes).clone()}
                    on_protocol_click={on_protocol_click}
                />

                <TracerouteSection nodes={(*nodes).clone()} />
                <RouteLookup nodes={(*nodes).clone()} on_lookup={on_route_lookup} />

                <ContentModal
                    visible={*modal_active}
                    content={(*modal_content).clone()}
                    command={(*modal_command).clone()}
                    on_close={
                        let modal_active = modal_active.clone();
                        Callback::from(move |_| {
                            modal_active.set(false);
                        })
                    }
                />
            </div>
        </main>
    }
}

fn handle_protocol_click(
    node: String,
    proto: String,
    modal_active: UseStateHandle<bool>,
    protocol_state: UseStateHandle<String>,
    modal_command: UseStateHandle<Option<String>>,
) {
    modal_active.set(true);
    protocol_state.set("Loading...".to_string());
    modal_command.set(Some(format!(
        "{}@{}$ birdc show protocols all {}",
        username(),
        node,
        proto
    )));

    spawn_local(async move {
        let stream_handle = protocol_state.clone();
        let url = backend_api(&format!("/api/node/{}/protocol/{}", node, proto));
        let mut aggregated = String::new();
        let result = stream_fetch(url, {
            let stream_state = stream_handle.clone();
            move |chunk| {
                aggregated.push_str(&chunk);
                let filtered = filter_protocol_details(&aggregated);
                stream_state.set(filtered);
            }
        })
        .await;

        if let Err(err) = result {
            stream_handle.set(format!("Failed to load protocol details: {}", err));
        }
    });
}

fn handle_route_lookup(
    node: String,
    target: String,
    all: bool,
    modal_active: UseStateHandle<bool>,
    protocol_state: UseStateHandle<String>,
    modal_command: UseStateHandle<Option<String>>,
) {
    modal_active.set(true);
    protocol_state.set("Loading...".to_string());
    let all_flag = if all { " all" } else { "" };
    modal_command.set(Some(format!(
        "{}@{}$ birdc show route for {}{}",
        username(),
        node,
        target,
        all_flag
    )));

    spawn_local(async move {
        let stream_handle = protocol_state.clone();
        let url = backend_api(&format!(
            "/api/route?node={}&target={}&all={}",
            node, target, all
        ));
        let mut aggregated = String::new();
        let result = stream_fetch(url, {
            let stream_state = stream_handle.clone();
            move |chunk| {
                aggregated.push_str(&chunk);
                stream_state.set(aggregated.clone());
            }
        })
        .await;

        if let Err(err) = result {
            stream_handle.set(format!("Failed to load route details: {}", err));
        }
    });
}

async fn fetch_status(
    nodes: &UseStateHandle<Vec<NodeStatus>>,
    fetch_error: &UseStateHandle<Option<String>>,
    data_ready: &UseStateHandle<bool>,
    last_known_updates: &mut HashMap<String, DateTime<Utc>>,
) {
    match Request::get(&backend_api("/api/status")).send().await {
        Ok(resp) if resp.ok() => match resp.json::<Vec<NodeStatus>>().await {
            Ok(mut data) => {
                for node in data.iter_mut() {
                    if let Some(err) = &node.error {
                        log_error(&format!("Node {} error: {}", node.name, err));
                        if let Some(previous) = last_known_updates.get(&node.name) {
                            node.last_updated = *previous;
                        }
                    } else {
                        last_known_updates.insert(node.name.clone(), node.last_updated);
                    }
                }

                fetch_error.set(None);
                if !data.is_empty() {
                    data_ready.set(true);
                }
                nodes.set(data);
            }
            Err(err) => {
                let msg = format!("Failed to parse backend response: {}", err);
                fetch_error.set(Some(msg.clone()));
                log_error(&msg);
            }
        },
        Ok(resp) => {
            let msg = format!("Status fetch failed with HTTP {}", resp.status());
            fetch_error.set(Some(msg.clone()));
            log_error(&msg);
        }
        Err(err) => {
            let msg = format!("Status fetch request error: {}", err);
            fetch_error.set(Some(msg.clone()));
            log_error(&msg);
        }
    }
}
