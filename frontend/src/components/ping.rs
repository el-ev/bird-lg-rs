
use common::{models::NodeProtocol, utils::validate_target};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::shell::{ShellButton, ShellInput, ShellPrompt, ShellSelect};
use crate::{
    services::api::perform_ping,
    store::{AppEvent, LgStateHandle, ping::PingAction, route_info::RouteInfoHandle},
};

#[function_component(Ping)]
pub fn ping() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let ping_state = &state.ping;

    let nodes: Vec<NodeProtocol> = if let Some(node) = &route_info.node_info {
        vec![node.clone()]
    } else {
        state.nodes.clone()
    };

    let on_ping = {
        let state = state.clone();
        Callback::from(move |(node, target, version): (String, String, String)| {
            state.dispatch(crate::store::AppEvent::Ping(PingAction::SetLastParams(
                target.clone(),
                version.clone(),
            )));
            perform_ping(&state, node, target, version);
        })
    };

    let on_node_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            state.dispatch(AppEvent::Ping(PingAction::SetNode(target.value())));
        })
    };

    let on_version_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target_unchecked_into();
            state.dispatch(AppEvent::Ping(PingAction::SetVersion(target.value())));
        })
    };

    let on_target_change = {
        let state = state.clone();
        Callback::from(move |value: String| {
            state.dispatch(AppEvent::Ping(PingAction::SetTarget(value)));
        })
    };

    let on_submit = {
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let target = state.ping.target.clone().trim().to_string();
            
            if let Err(err) = validate_target(&target) {
                state.dispatch(AppEvent::Ping(PingAction::SetError(err)));
                return;
            }
            state.dispatch(AppEvent::Ping(PingAction::ClearError));
            state.dispatch(AppEvent::Ping(PingAction::SetLastParams(
                target.clone(),
                state.ping.version.clone(),
            )));

            state.dispatch(AppEvent::Ping(PingAction::Start));

            let validated_target = target;
            let ping_node = state.ping.node.clone();
            let ping_version = state.ping.version.clone();

            on_ping.emit((ping_node, validated_target, ping_version));
        })
    };

    html! {
        <section>
            <h3>{"Ping"}</h3>
            <form class="shell-line" onsubmit={on_submit}>
                <ShellPrompt>
                    {format!("{}@", state.username)}
                    <ShellSelect
                        value={ping_state.node.clone()}
                        on_change={on_node_change}
                    >
                        { for nodes.iter().enumerate().map(|(i, n)| html! {
                            <option value={n.name.clone()} selected={i == 0}>{ &n.name }</option>
                        }) }
                    </ShellSelect>
                    {"$ "}
                </ShellPrompt>
                { "ping -c 5 " }
                <ShellSelect
                    value={ping_state.version.clone()}
                    on_change={on_version_change}
                >
                    <option value="" selected=true>{"  "}</option>
                    <option value="4">{"-4"}</option>
                    <option value="6">{"-6"}</option>
                </ShellSelect>
                <span>{ " " }</span>
                <ShellInput
                    value={ping_state.target.clone()}
                    on_change={on_target_change}
                    placeholder="<target>"
                />
                <ShellButton type_="submit" text="↵" />
            </form>
            {
                if let Some(err) = &ping_state.error {
                    html! { <div class="error-message">{ err }</div> }
                } else {
                    html! {}
                }
            }
        </section>
    }
}
