use std::net::IpAddr;

use ipnet::IpNet;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::components::shell::{ShellButton, ShellInput, ShellPrompt, ShellSelect, ShellToggle};
use crate::services::stream_fetch;
use crate::store::modal::ModalAction;
use crate::store::{Action, AppState};

#[derive(Properties, PartialEq)]
pub struct RouteLookupProps {
    pub state: UseReducerHandle<AppState>,
    pub nodes: Vec<crate::models::NodeStatus>,
    pub on_lookup: Callback<(String, String, bool)>,
}

#[function_component(RouteLookup)]
pub fn route_lookup(props: &RouteLookupProps) -> Html {
    let selected_node = use_state(String::new);
    let target = use_state(String::new);
    let all = use_state(|| false);
    let error = use_state(|| None::<String>);

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
        let on_lookup = props.on_lookup.clone();
        let nodes = props.nodes.clone();

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

            on_lookup.emit((final_node, target_val, all_val));
        })
    };

    html! {
        <section>
            <h3>{"Route Lookup"}</h3>
            <form class="shell-line" onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", props.state.username)}
                    <ShellSelect
                        value={(*selected_node).clone()}
                        on_change={on_node_change}
                    >
                        { for props.nodes.iter().map(|n| html! {
                            <option value={n.name.clone()}>{ &n.name }</option>
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
                <ShellToggle
                    active={*all}
                    on_toggle={on_all_toggle}
                    label="all"
                />
                <ShellButton type_="submit" text="↵" />
            </form>
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

pub fn handle_route_lookup(
    node: String,
    target: String,
    all: bool,
    state: UseReducerHandle<AppState>,
) {
    let command = if all {
        format!(
            "{}@{}$ birdc show route {} all",
            state.username, node, target
        )
    } else {
        format!("{}@{}$ birdc show route {}", state.username, node, target)
    };

    state.dispatch(Action::Modal(ModalAction::Open {
        content: "Loading...".to_string(),
        command: Some(command),
    }));

    spawn_local(async move {
        let url = format!(
            "{}/api/routes/{}?target={}&all={}",
            state.backend_url, node, target, all
        );

        let mut aggregated = String::new();
        let result = stream_fetch(url, {
            let state = state.clone();
            move |chunk| {
                aggregated.push_str(&chunk);
                state.dispatch(Action::Modal(ModalAction::UpdateContent(
                    aggregated.clone(),
                )));
            }
        })
        .await;

        if let Err(err) = result {
            state.dispatch(Action::Modal(ModalAction::UpdateContent(format!(
                "Failed to load route details: {}",
                err
            ))));
        }
    });
}
