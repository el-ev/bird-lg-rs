use std::{cmp::Ordering, net::Ipv4Addr};
use yew::prelude::*;

use crate::store::AppStateHandle;

#[function_component(PeeringPage)]
pub fn peering_page() -> Html {
    let state = use_context::<AppStateHandle>().expect("no ctx found");

    let content = match &state.network_info {
        Some(network_info) => {
            let mut peers = network_info.peering.iter().collect::<Vec<_>>();
            peers.sort_by(|(node_a, info_a), (node_b, info_b)| {
                match (info_a.ipv4.as_deref(), info_b.ipv4.as_deref()) {
                    (Some(a), Some(b)) => match (a.parse::<Ipv4Addr>(), b.parse::<Ipv4Addr>()) {
                        (Ok(a_ip), Ok(b_ip)) => a_ip.cmp(&b_ip),
                        _ => a.cmp(b),
                    },
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => node_a.cmp(node_b),
                }
            });

            html! {
                <>
                    <article class="peering-card">
                        <div class="peering-card-header">
                            <h3 class="peering-card-title">{"Prefix"}</h3>
                        </div>
                        <dl class="peering-grid">
                            {
                                if !network_info.ipv4_prefix.is_empty() {
                                    let ipv4_prefixes = network_info.ipv4_prefix.join(", ");
                                    html! {
                                        <>
                                            <dt class="peering-label">{"IPv4"}</dt>
                                            <dd class="peering-value">{ipv4_prefixes}</dd>
                                        </>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            {
                                if !network_info.ipv6_prefix.is_empty() {
                                    let ipv6_prefixes = network_info.ipv6_prefix.join(", ");
                                    html! {
                                        <>
                                            <dt class="peering-label">{"IPv6"}</dt>
                                            <dd class="peering-value">{ipv6_prefixes}</dd>
                                        </>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </dl>
                    </article>

                    <div>
                        <div class="peering-card-header">
                            <h3 class="peering-card-title">{"Nodes"}</h3>
                            {
                                if let Some(comment) = &network_info.comment {
                                    html! { <p class="peering-card-subtitle">{ render_multiline_text(comment) }</p> }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                        <div class="peering-node-list">
                            { for peers.into_iter().map(|(node, info)| {
                                let node_name = node.clone();
                                html! {
                                    <article class="peering-card peering-node">
                                        <div class="peering-node-header">
                                            <h4 class="peering-node-title">{node_name}</h4>
                                            if let Some(comment) = &info.comment {
                                                <span class="peering-node-meta">{comment}</span>
                                            }
                                        </div>
                                        <dl class="peering-grid">
                                            {
                                                if let Some(endpoint) = &info.endpoint {
                                                    html! {
                                                        <>
                                                            <dt class="peering-label">{"Endpoint"}</dt>
                                                            <dd class="peering-value">{endpoint}</dd>
                                                        </>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(ipv4) = &info.ipv4 {
                                                    html! {
                                                        <>
                                                            <dt class="peering-label">{"IPv4"}</dt>
                                                            <dd class="peering-value">{ipv4}</dd>
                                                        </>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(ipv6) = &info.ipv6 {
                                                    html! {
                                                        <>
                                                            <dt class="peering-label">{"IPv6"}</dt>
                                                            <dd class="peering-value">{ipv6}</dd>
                                                        </>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(link_local) = &info.link_local_ipv6 {
                                                    html! {
                                                        <>
                                                            <dt class="peering-label">{"IPv6 Link-Local"}</dt>
                                                            <dd class="peering-value">{link_local}</dd>
                                                        </>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if info.wg_pubkey.is_some() {
                                                    html! {
                                                        <>
                                                            <dt class="peering-label">{"Tunnel"}</dt>
                                                            <dd class="peering-value">{"WireGuard"}</dd>
                                                        </>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(wg_pubkey) = &info.wg_pubkey {
                                                    html! {
                                                        <>
                                                            <dt class="peering-label">{"WG Public Key"}</dt>
                                                            <dd class="peering-value">{wg_pubkey}</dd>
                                                        </>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </dl>
                                    </article>
                                }
                            }) }
                        </div>
                    </div>

                    {
                        if !network_info.contacts.is_empty() {
                            let mut contacts = network_info.contacts.iter().collect::<Vec<_>>();
                            contacts.sort_by(|(a, _), (b, _)| a.cmp(b));
                            html! {
                                <article class="peering-card peering-contact">
                                    <div class="peering-card-header">
                                        <h3 class="peering-card-title">{"Contact"}</h3>
                                    </div>
                                    <dl class="peering-grid peering-contact-grid">
                                        { for contacts.into_iter().map(|(label, value)| {
                                            html! {
                                                <>
                                                    <dt class="peering-label peering-contact-label">{label}</dt>
                                                    <dd class="peering-value peering-contact-value">{ render_multiline_text(value) }</dd>
                                                </>
                                            }
                                        }) }
                                    </dl>
                                </article>
                            }
                        } else {
                            html! {}
                        }
                    }
                </>
            }
        }
        None => html! {
            <div class="peering-empty">{"Loading peering information..."}</div>
        },
    };

    html! {
        <section class="peering">
            {content}
        </section>
    }
}

fn render_multiline_text(text: &str) -> Html {
    let lines: Vec<&str> = text.split('\n').collect();
    html! {
        <>
            { for lines.iter().enumerate().map(|(idx, line)| {
                let needs_break = idx + 1 < lines.len();
                html! {
                    <>
                        { *line }
                        { if needs_break { html! { <br/> } } else { html! {} } }
                    </>
                }
            }) }
        </>
    }
}
