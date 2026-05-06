use std::{cell::Cell, rc::Rc};

use ui_components::shell::ShellLine;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use super::data_table::{DataTable, TableRow};
use crate::{
    services::api::request_wireguard,
    store::{LgStateHandle, route_info::RouteInfoHandle},
    utils::sleep_ms,
};

const REFRESH_INTERVAL_MS: i32 = 30_000;

#[function_component(WireGuard)]
pub fn wireguard_section() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let node_route = route_info.node_name.is_some();

    let wireguard_data = route_info.scoped_wireguard_nodes(state.wireguard.as_slice());

    {
        let state = state.clone();
        use_effect(move || {
            let active = Rc::new(Cell::new(true));
            let cleanup = active.clone();
            spawn_local(async move {
                loop {
                    sleep_ms(REFRESH_INTERVAL_MS).await;
                    if !active.get() {
                        break;
                    }
                    if let Err(error) = request_wireguard(&state).await {
                        tracing::error!("Failed to refresh WireGuard data: {}", error);
                    }
                    if !active.get() {
                        break;
                    }
                }
            });
            move || cleanup.set(false)
        });
    }

    html! {
        <section>
            <h3>{"WireGuard"}</h3>
            <div>
                {
                    if node_route && wireguard_data.is_empty() {
                        html! { <p class="status-message">{"No WireGuard data loaded for this node"}</p> }
                    } else {
                        html! {
                            for wireguard_data.iter().map(|node_wg| {
                                html! {
                                    <details key={node_wg.name.clone()} class="expandable-item" open=true>
                                        <summary class="summary-header">
                                            <h4 class="item-title">{ &node_wg.name }</h4>
                                        </summary>
                                       {
                                            if let Some(err) = &node_wg.error {
                                                html! { <pre class="status-message--error">{ err }</pre> }
                                            } else if node_wg.peers.is_empty() {
                                                html! { <p class="status-message">{"No WireGuard peers found"}</p> }
                                            } else {
                                                html! {
                                                    <>
                                                        <ShellLine
                                                            prompt={format!("{}@{}$ ", state.username, &node_wg.name)}
                                                            command={"wg show".to_string()}
                                                            style={"font-size: 0.9em;".to_string()}
                                                        />
                                                        <DataTable
                                                            headers={
                                                                [
                                                                    "Peer",
                                                                    "Latest Handshake",
                                                                    "Transfer RX",
                                                                    "Transfer TX",
                                                                ]
                                                                .map(AttrValue::from)
                                                                .to_vec()
                                                            }
                                                            rows={
                                                                node_wg.peers.iter().map(|peer| {
                                                                    TableRow {
                                                                        cells: vec![
                                                                            html! { &peer.name },
                                                                            html! { peer.latest_handshake.as_deref().unwrap_or("never") },
                                                                            html! { &peer.transfer_rx },
                                                                            html! { &peer.transfer_tx },
                                                                        ],
                                                                        on_click: None,
                                                                    }
                                                                })
                                                                .collect::<Vec<_>>()
                                                            }
                                                        />
                                                    </>
                                                }
                                            }
                                        }
                                    </details>
                                }
                            })
                        }
                    }
                }
            </div>
        </section>
    }
}
