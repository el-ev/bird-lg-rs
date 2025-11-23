use std::collections::HashMap;

use chrono::{DateTime, Utc};

use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::{
    node_list::NodeList, protocol_modal::ProtocolModal, status_banner::StatusBanner,
    traceroute::TracerouteSection,
};
use crate::config::backend_api;
use crate::models::NodeStatus;
use crate::services::{log_error, sleep_ms, stream_fetch};
use crate::utils::filter_protocol_details;

#[function_component(App)]
pub fn app() -> Html {
    let nodes = use_state(Vec::<NodeStatus>::new);
    let selected_protocol = use_state(|| None::<(String, String)>);
    let protocol_details = use_state(String::new);
    let fetch_error = use_state(|| None::<String>);
    let data_ready = use_state(|| false);

    {
        let nodes = nodes.clone();
        let fetch_error = fetch_error.clone();
        let data_ready = data_ready.clone();
        use_effect_with((), move |_| {
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
        });
    }

    let on_protocol_click = {
        let selected_protocol = selected_protocol.clone();
        let protocol_details = protocol_details.clone();
        Callback::from(move |(node, proto): (String, String)| {
            let selected_protocol = selected_protocol.clone();
            let protocol_state = protocol_details.clone();
            selected_protocol.set(Some((node.clone(), proto.clone())));
            protocol_state.set("Loading...".to_string());

            spawn_local(async move {
                let url = backend_api(&format!("/api/node/{}/protocol/{}", node, proto));

                let mut aggregated = String::new();
                let stream_handle = protocol_state.clone();
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
        })
    };

    let waiting_for_data = nodes.is_empty() && !*data_ready;

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

                <ProtocolModal
                    visible={selected_protocol.is_some()}
                    content={(*protocol_details).clone()}
                    on_close={
                        let selected_protocol = selected_protocol.clone();
                        Callback::from(move |_| selected_protocol.set(None))
                    }
                />

                <TracerouteSection nodes={(*nodes).clone()} />
            </div>
        </main>
    }
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
