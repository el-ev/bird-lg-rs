use yew::prelude::*;

use crate::{
    components::cards::{ContactCard, PeeringField, PeeringNodeCard},
    store::AppStateHandle,
};

#[function_component(PeeringPage)]
pub fn peering_page() -> Html {
    let state = use_context::<AppStateHandle>().expect("no ctx found");

    if state.network_info.is_none() {
        return html! {};
    }
    let network_info = state.network_info.as_ref().unwrap();

    let ipv4_prefixes = network_info.ipv4_prefix.join(", ");
    let ipv6_prefixes = network_info.ipv6_prefix.join(", ");

    let show_prefix_card = !ipv4_prefixes.is_empty() || !ipv6_prefixes.is_empty();
    let prefix_card = html! {
        <article class="peering-card">
            <div class="peering-card-header">
                <h3 class="peering-card-title">{"Prefix"}</h3>
            </div>
            <dl class="peering-grid">
                if !ipv4_prefixes.is_empty() {
                    <PeeringField label="IPv4" value={ipv4_prefixes} />
                }
                if !ipv6_prefixes.is_empty() {
                    <PeeringField label="IPv6" value={ipv6_prefixes} />
                }
            </dl>
        </article>
    };

    let content = match &state.network_info {
        Some(network_info) => {
            let mut peers = network_info.peering.iter().collect::<Vec<_>>();
            peers.sort_by(|(_, info_a), (_, info_b)| info_a.ipv4.cmp(&info_b.ipv4));

            html! {
                <>
                    if show_prefix_card {
                        {prefix_card}
                    }

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
                                html! {
                                    <PeeringNodeCard node_name={node.clone()} node_info={info.clone()} />
                                }
                            }) }
                        </div>
                    </div>

                    <ContactCard contacts={
                        network_info
                            .contacts
                            .iter()
                            .map(|(label, value)| (label.clone(), value.clone()))
                            .collect::<Vec<_>>()
                    } />
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
