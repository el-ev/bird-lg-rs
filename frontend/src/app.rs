use std::collections::HashMap;

use chrono::{DateTime, Utc};
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::node_list::handle_protocol_click;
use crate::components::route_lookup::handle_route_lookup;
use crate::components::{
    content_modal::ContentModal, node_list::NodeList, route_lookup::RouteLookup,
    status_banner::StatusBanner, traceroute::Traceroute,
};
use crate::config::load_config;
use crate::models::{NetworkInfo, NodeStatus};
use crate::services::{log_error, sleep_ms};
use crate::store::modal::ModalAction;
use crate::store::{Action, AppState};

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(AppState::default);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match load_config().await {
                    Ok(config) => {
                        state.dispatch(Action::SetConfig {
                            username: config.username.clone(),
                            backend_url: config.backend_url.clone(),
                        });
                        state.dispatch(Action::SetConfigReady(true));
                    }
                    Err(err) => {
                        let message = format!("Configuration load failed: {}", err);
                        state.dispatch(Action::SetError(message.clone()));
                        log_error(&message);
                    }
                }
            });

            || ()
        });
    }

    // Start background tasks once config is ready
    {
        let state = state.clone();
        use_effect_with(state.config_ready, move |ready| {
            if *ready {
                let state_poll = state.clone();
                let backend_url = state.backend_url.clone();
                spawn_local(async move {
                    let mut last_known_updates = HashMap::new();
                    loop {
                        fetch_status(&state_poll, &mut last_known_updates, &backend_url).await;
                        sleep_ms(5000).await;
                    }
                });

                let state_info = state.clone();
                let backend_url2 = state.backend_url.clone();
                spawn_local(async move {
                    fetch_network_info(&state_info, &backend_url2).await;
                });
            }

            || ()
        });
    }

    let on_protocol_click = {
        let state = state.clone();
        Callback::from(move |(node, proto): (String, String)| {
            handle_protocol_click(node, proto, state.clone());
        })
    };

    let on_route_lookup = {
        let state = state.clone();
        Callback::from(move |(node, target, all): (String, String, bool)| {
            handle_route_lookup(node, target, all, state.clone());
        })
    };

    let waiting_for_data = state.nodes.is_empty() && (!state.data_ready || !state.config_ready);

    html! {
        <main class="hero">
            <div class="container">
                <h2 class="title title-flex">
                    {"Looking Glass"}
                    {
                        if let Some(ref info) = state.network_info {
                            html! {
                                <span class="title-footnote">
                                    { " of " } { &info.name } { " " } { &info.asn } {" on DN42 "}
                                </span>
                            }
                        } else {
                            html! {}
                        }
                    }
                </h2>

                <StatusBanner
                    fetch_error={state.fetch_error.clone()}
                    waiting_for_data={waiting_for_data}
                />

                if state.config_ready {
                    <>
                        <NodeList
                            state={state.clone()}
                            on_protocol_click={on_protocol_click}
                        />

                        <Traceroute state={state.clone()} />
                        <RouteLookup
                            state={state.clone()}
                            on_lookup={on_route_lookup} />

                        <ContentModal
                            visible={state.modal.active}
                            content={state.modal.content.clone()}
                            command={state.modal.command.clone()}
                            on_close={
                                let state = state.clone();
                                Callback::from(move |_| {
                                    state.dispatch(Action::Modal(ModalAction::Close));
                                })
                            }
                        />
                    </>
                }
            </div>
        </main>
    }
}

async fn fetch_status(
    state: &UseReducerHandle<AppState>,
    last_known_updates: &mut HashMap<String, DateTime<Utc>>,
    backend_url: &str,
) {
    match Request::get(&format!(
        "{}/api/protocols",
        backend_url.trim_end_matches('/')
    ))
    .send()
    .await
    {
        Ok(resp) if resp.ok() => match resp.json::<Vec<NodeStatus>>().await {
            Ok(data) => {
                for new_node in &data {
                    if let Some(old_time) = last_known_updates.get(&new_node.name)
                        && &new_node.last_updated == old_time
                    {
                        continue;
                    }
                    last_known_updates.insert(new_node.name.clone(), new_node.last_updated);
                }

                state.dispatch(Action::ClearError);
                if !data.is_empty() {
                    state.dispatch(Action::SetDataReady(true));
                }
                state.dispatch(Action::SetNodes(data));
            }
            Err(err) => {
                let msg = format!("Failed to parse backend response: {}", err);
                state.dispatch(Action::SetError(msg.clone()));
                log_error(&msg);
            }
        },
        Ok(resp) => {
            let msg = format!("Status fetch failed with HTTP {}", resp.status());
            state.dispatch(Action::SetError(msg.clone()));
            log_error(&msg);
        }
        Err(err) => {
            let msg = format!("Status fetch request error: {}", err);
            state.dispatch(Action::SetError(msg.clone()));
            log_error(&msg);
        }
    }
}

async fn fetch_network_info(state: &UseReducerHandle<AppState>, backend_url: &str) {
    match Request::get(&format!("{}/api/info", backend_url.trim_end_matches('/')))
        .send()
        .await
    {
        Ok(resp) if resp.ok() => match resp.json::<Option<NetworkInfo>>().await {
            Ok(info) => {
                state.dispatch(Action::SetNetworkInfo(info));
            }
            Err(err) => {
                log_error(&format!("Failed to parse network info: {}", err));
            }
        },
        Ok(resp) => {
            log_error(&format!(
                "Network info fetch failed with HTTP {}",
                resp.status()
            ));
        }
        Err(err) => {
            log_error(&format!("Network info request error: {}", err));
        }
    }
}
