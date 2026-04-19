use common::{models::NodeProtocol, utils::validate_target};
use ui_components::shell::{ShellButton, ShellForm, ShellInput, ShellPrompt, ShellSelect};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    services::api::perform_ping,
    store::{LgStateHandle, route_info::RouteInfoHandle},
};

#[function_component(Ping)]
pub fn ping() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let selected_node = use_state(String::new);
    let target = use_state(String::new);
    let version = use_state(String::new);
    let error = use_state(|| None::<String>);

    let nodes: Vec<NodeProtocol> = route_info.scoped_protocol_nodes(state.nodes.as_slice());

    let selected_node_value = if selected_node.is_empty() {
        nodes
            .first()
            .map(|node| node.name.clone())
            .unwrap_or_default()
    } else {
        (*selected_node).clone()
    };

    let on_node_change = {
        let selected_node = selected_node.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            selected_node.set(target.value());
        })
    };

    let on_version_change = {
        let version = version.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            version.set(target.value());
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

    let on_submit = {
        let error = error.clone();
        let node = selected_node.clone();
        let target = target.clone();
        let version = version.clone();
        let state = state.clone();
        let nodes = nodes.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let target_value = (*target).trim().to_string();
            if let Err(err) = validate_target(&target_value) {
                error.set(Some(err));
                return;
            }

            let node_value = if node.is_empty() {
                nodes
                    .first()
                    .map(|entry| entry.name.clone())
                    .unwrap_or_default()
            } else {
                (*node).clone()
            };

            error.set(None);
            let version_value = (*version).clone();
            let state_async = state.clone();

            spawn_local(async move {
                if let Err(fetch_error) =
                    perform_ping(&state_async, node_value, target_value, version_value).await
                {
                    tracing::error!("Ping request failed: {}", fetch_error);
                }
            });
        })
    };

    html! {
        <section>
            <h3>{"Ping"}</h3>
            <ShellForm onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", state.username)}
                    <ShellSelect value={selected_node_value} on_change={on_node_change}>
                        { for nodes.iter().enumerate().map(|(i, node)| html! {
                            <option value={node.name.clone()} selected={i == 0}>{ &node.name }</option>
                        }) }
                    </ShellSelect>
                    {"$ "}
                </ShellPrompt>
                { "ping -c 5 " }
                <ShellSelect value={(*version).clone()} on_change={on_version_change}>
                    <option value="" selected=true>{"  "}</option>
                    <option value="4">{"-4"}</option>
                    <option value="6">{"-6"}</option>
                </ShellSelect>
                <span>{ " " }</span>
                <ShellInput
                    value={(*target).clone()}
                    on_change={on_target_change}
                    placeholder="<target>"
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
