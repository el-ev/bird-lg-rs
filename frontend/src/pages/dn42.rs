use std::collections::HashSet;

use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    routes::Route,
    store::LgStateHandle,
    utils::{fetch_json, get_hostname, is_dn42_domain},
};

#[derive(Clone, Default, Deserialize, PartialEq)]
struct VisitorInfo {
    #[serde(default)]
    node: String,
    #[serde(default)]
    ip: String,
}

#[function_component(Dn42Page)]
pub fn dn42_page() -> Html {
    if !is_dn42_domain() {
        return html! { <Redirect<Route> to={Route::Root} /> };
    }

    let state = use_context::<LgStateHandle>().expect("no ctx found");
    let visitor_info = use_state(VisitorInfo::default);

    {
        let visitor_info = visitor_info.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(info) = fetch_json::<VisitorInfo>("/visitor-info").await {
                    visitor_info.set(info);
                }
            });
        });
    }

    let domain = get_hostname();

    let asn = state
        .network_info
        .as_ref()
        .map(|info| info.asn.clone())
        .unwrap_or_else(|| "?".to_string());

    let node_count = state.nodes.len();
    let session_count: usize = state.nodes.iter().map(|n| n.protocols.len()).sum();
    let peer_count: usize = state
        .nodes
        .iter()
        .flat_map(|n| n.protocols.iter())
        .filter_map(|p| p.name.strip_prefix("dn42_"))
        .collect::<HashSet<_>>()
        .len();

    let node_display = if visitor_info.node.is_empty() {
        "..."
    } else {
        &visitor_info.node
    };
    let ip_display = if visitor_info.ip.is_empty() {
        "..."
    } else {
        &visitor_info.ip
    };

    html! {
        <section class="dn42-page">
            <article class="dn42-card dn42-visitor-card">
                <p class="dn42-visitor-line">
                    {"You've reached "}
                    <span class="dn42-highlight">{&domain}</span>
                    {" on "}
                    <span class="dn42-highlight">{&asn}</span>
                    {"."}
                </p>
                <p class="dn42-visitor-line">
                    {"Served by node: "}
                    <span class="dn42-highlight">{node_display}</span>
                </p>
                <p class="dn42-visitor-line">
                    {"Your address: "}
                    <span class="dn42-highlight">{ip_display}</span>
                </p>
            </article>

            <article class="dn42-card dn42-stats-card">
                <p class="dn42-stats-line">
                    {"Monitoring "}
                    <strong>{session_count}</strong>
                    {" sessions with "}
                    <strong>{peer_count}</strong>
                    {" peers across "}
                    <strong>{node_count}</strong>
                    {" nodes"}
                </p>
            </article>

            <nav class="dn42-links">
                <Link<Route> to={Route::Protocols} classes="dn42-link-card">
                    {"Protocols"}
                </Link<Route>>
                <Link<Route> to={Route::Peering} classes="dn42-link-card">
                    {"Peering"}
                </Link<Route>>
                <a href={format!("https://explorer.burble.dn42/?#/aut-num/{asn}")} target="_blank" rel="noopener" class="dn42-link-card">
                    {"Registry"}
                </a>
            </nav>

        </section>
    }
}
