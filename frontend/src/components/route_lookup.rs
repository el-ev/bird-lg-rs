use std::net::IpAddr;

use common::models::NodeProtocol;
use ipnet::IpNet;
use ui_components::shell::{
    ShellButton, ShellForm, ShellInput, ShellPlainToggle, ShellPrompt, ShellSelect,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    services::api::perform_route_lookup,
    store::{LgStateHandle, route_info::RouteInfoHandle},
};

#[function_component(RouteLookup)]
pub fn route_lookup() -> Html {
    let selected_node = use_state(String::new);
    let target = use_state(String::new);
    let all = use_state(|| false);
    let error = use_state(|| None::<String>);
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let nodes: Vec<NodeProtocol> = route_info.scoped_protocol_nodes(state.nodes.as_slice());

    let on_node_change = {
        let selected_node = selected_node.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            selected_node.set(target.value());
        })
    };

    let on_target_change = {
        let target = target.clone();
        let error = error.clone();
        Callback::from(move |value: String| {
            target.set(value);
            error.set(None);
        })
    };

    let on_all_toggle = {
        let all = all.clone();
        Callback::from(move |_| all.set(!*all))
    };

    let on_submit = {
        let selected_node = selected_node.clone();
        let target = target.clone();
        let all = all.clone();
        let error = error.clone();
        let state = state.clone();
        let nodes = nodes.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let node_val = (*selected_node).clone();
            let target_val = (*target).trim().to_string();
            let all_val = *all;

            if target_val.is_empty() {
                error.set(Some("Target is required".to_string()));
                return;
            }

            if target_val.parse::<IpAddr>().is_err() && target_val.parse::<IpNet>().is_err() {
                error.set(Some("Invalid IP or CIDR".to_string()));
                return;
            }

            let final_node = if node_val.is_empty() {
                if let Some(first) = nodes.first() {
                    first.name.clone()
                } else {
                    error.set(Some("No nodes available".to_string()));
                    return;
                }
            } else {
                node_val
            };

            let state = state.clone();
            spawn_local(async move {
                if let Err(error) =
                    perform_route_lookup(&state, final_node, target_val, all_val).await
                {
                    tracing::error!("Route lookup failed: {}", error);
                }
            });
        })
    };

    html! {
        <section>
            <h3>{"Route Lookup"}</h3>
            <ShellForm onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", state.username)}
                    <ShellSelect
                        value={(*selected_node).clone()}
                        on_change={on_node_change}
                    >
                        { for nodes.iter().enumerate().map(|(i, n)| html! {
                            <option value={n.name.clone()} selected={i == 0}>{ &n.name }</option>
                        }) }
                    </ShellSelect>
                    {"$ "}
                </ShellPrompt>
                { "birdc show route for " }
                <ShellInput
                    value={(*target).clone()}
                    on_change={on_target_change}
                    placeholder="<ip>[/<mask>]"
                />
                <span>{ " " }</span>
                <ShellPlainToggle
                    active={*all}
                    on_toggle={on_all_toggle}
                    label="all"
                />
                <ShellButton type_="submit" text="↵" class="shell-button--submit" />
            </ShellForm>
            {
                if let Some(err) = &*error {
                    html! { <div class="error-message">{ err }</div> }
                } else {
                    html! {}
                }
            }
        </section>
    }
}
