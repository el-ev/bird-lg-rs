use std::collections::HashMap;

use reqwasm::http::Request;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::prelude::*;

use crate::components::{
    node_list::NodeList, protocol_modal::ProtocolModal, status_banner::StatusBanner,
    traceroute::TracerouteSection,
};
use crate::config::backend_api;
use crate::models::NodeStatus;
use crate::services::stream_fetch;
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
            let nodes = nodes.clone();
            let fetch_error = fetch_error.clone();
            let data_ready = data_ready.clone();
            spawn_local(async move {
                loop {
                    let resp = Request::get(&backend_api("/api/status")).send().await;

                    match resp {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<Vec<NodeStatus>>().await {
                                    Ok(mut data) => {
                                        let previous_nodes = (*nodes).clone();
                                        let previous_map: HashMap<String, _> = previous_nodes
                                            .into_iter()
                                            .map(|n| (n.name.clone(), n.last_updated))
                                            .collect();

                                        for node in data.iter_mut() {
                                            if let (Some(_), Some(prev)) =
                                                (node.error.as_ref(), previous_map.get(&node.name))
                                            {
                                                node.last_updated = *prev;
                                            }
                                        }

                                        for node in data.iter() {
                                            if let Some(err) = &node.error {
                                                console::error_1(&JsValue::from_str(&format!(
                                                    "Node {} error: {}",
                                                    node.name, err
                                                )));
                                            }
                                        }

                                        fetch_error.set(None);
                                        if !data.is_empty() {
                                            data_ready.set(true);
                                        }
                                        nodes.set(data);
                                    }
                                    Err(err) => {
                                        let msg =
                                            format!("Failed to parse backend response: {}", err);
                                        fetch_error.set(Some(msg.clone()));
                                        console::error_1(&JsValue::from_str(&msg));
                                    }
                                }
                            } else {
                                let msg =
                                    format!("Status fetch failed with HTTP {}", resp.status());
                                fetch_error.set(Some(msg.clone()));
                                console::error_1(&JsValue::from_str(&msg));
                            }
                        }
                        Err(err) => {
                            let msg = format!("Status fetch request error: {}", err);
                            fetch_error.set(Some(msg.clone()));
                            console::error_1(&JsValue::from_str(&msg));
                        }
                    }

                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        if let Some(window) = web_sys::window() {
                            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                &resolve, 5000,
                            );
                        }
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                }
            });
            || ()
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
