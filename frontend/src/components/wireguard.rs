use yew::prelude::*;

use super::data_table::{DataTable, TableRow};
use super::shell::ShellLine;

use crate::services::api::request_wireguard;
use crate::store::AppStateHandle;
use crate::store::route_info::RouteInfoHandle;

#[derive(Properties, PartialEq)]
pub struct WireGuardProps {
    #[prop_or(true)]
    pub default_open: bool,
}

#[function_component(WireGuard)]
pub fn wireguard_section(props: &WireGuardProps) -> Html {
    let state = use_context::<AppStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");

    let wireguard_data = if let Some(info) = &route_info.wireguard_info {
        std::slice::from_ref(info)
    } else {
        state.wireguard.as_slice()
    };

    let on_refresh = {
        let state = state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            request_wireguard(&state);
        })
    };

    html! {
        <section>
            <h3>
                {"WireGuard"}
                <button
                    class="refresh-button"
                    onclick={on_refresh}
                    title="Refresh WireGuard status"
                >
                    {"↻"}
                </button>
            </h3>
            <div>
                { for wireguard_data.iter().map(|node_wg| {
                    html! {
                        <details class="expandable-item" open={props.default_open}>
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
                                                                // html! { peer.endpoint.as_deref().unwrap_or("-") },
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
                }) }
            </div>
        </section>
    }
}
