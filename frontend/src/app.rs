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
use crate::config::{backend_api, load_config};
use crate::models::NodeStatus;
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
                    Ok(_) => state.dispatch(Action::SetConfigReady(true)),
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

    {
        let state = state.clone();
        use_effect_with(state.config_ready, move |ready| {
            if *ready {
                let state = state.clone();
                spawn_local(async move {
                    let mut last_known_updates = HashMap::new();
                    loop {
                        fetch_status(&state, &mut last_known_updates).await;
                        sleep_ms(5000).await;
                    }
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
                <h2 class="title title-flex">{"Looking Glass"}</h2>

                <StatusBanner
                    fetch_error={state.fetch_error.clone()}
                    waiting_for_data={waiting_for_data}
                />

                if state.config_ready {
                    <>
                        <NodeList
                            nodes={state.nodes.clone()}
                            on_protocol_click={on_protocol_click}
                        />

                        <Traceroute nodes={state.nodes.clone()} state={state.clone()} />
                        <RouteLookup nodes={state.nodes.clone()} on_lookup={on_route_lookup} />

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
