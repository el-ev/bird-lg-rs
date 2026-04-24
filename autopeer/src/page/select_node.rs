use ui_components::shell::ShellButton;
use yew::prelude::*;

use super::{humanize_ip_support, node_context_line, render_error};
use crate::{
    i18n::I18n,
    models::{NodeView, SessionState, SessionView, UiMessage},
};

#[derive(Properties, PartialEq)]
pub struct SelectNodeProps {
    pub i18n: I18n,
    pub loading: bool,
    pub nodes: Vec<NodeView>,
    pub sessions: Vec<SessionView>,
    pub selected_node_name: Option<String>,
    pub error: Option<UiMessage>,
    pub on_select_new_node: Callback<String>,
    pub on_select_edit_node: Callback<(String, SessionView)>,
    pub on_select_blocked: Callback<UiMessage>,
}

#[function_component(SelectNodePanel)]
pub fn select_node_panel(props: &SelectNodeProps) -> Html {
    let i18n = &props.i18n;

    html! {
        <article class="peering-card autopeer-panel">
            <div class="autopeer-panel-header">
                <p class="autopeer-panel-kicker">{i18n.t("stage1.kicker")}</p>
                <h3 class="autopeer-panel-title">{i18n.t("stage1.title")}</h3>
                <p class="text-secondary">
                    {i18n.t("stage1.description")}
                </p>
            </div>
            if props.nodes.is_empty() {
                <div class="autopeer-empty-state">
                    <p>{i18n.t("stage1.empty_title")}</p>
                    <p class="text-secondary">
                        {i18n.t("stage1.empty_body")}
                    </p>
                </div>
            } else {
                <div class="autopeer-node-grid">
                    {for props.nodes.iter().map(|node| {
                        let node_session = props.sessions.iter().find(|session| session.node == node.name).cloned();
                        let selected = props.selected_node_name.as_deref() == Some(node.name.as_str());
                        let autopeer_disabled = node.autopeer == Some(false);
                        let selectable = !autopeer_disabled && matches!(
                            node_session.as_ref().map(|session| &session.state),
                            None | Some(SessionState::Managed) | Some(SessionState::Manual) | Some(SessionState::StalledPr)
                        );
                        let state_label = if autopeer_disabled {
                            i18n.t("stage1.state.disabled")
                        } else {
                            node_session
                                .as_ref()
                                .map(|session| i18n.t(session.state.i18n_key()))
                                .unwrap_or(i18n.t("stage1.state.available"))
                        };
                        let state_note = if autopeer_disabled {
                            i18n.t("stage1.state.note.disabled")
                        } else {
                            match node_session.as_ref().map(|session| &session.state) {
                                None => i18n.t("stage1.state.note.create"),
                                Some(SessionState::Managed) => i18n.t("stage1.state.note.managed"),
                                Some(SessionState::Manual) => i18n.t("stage1.state.note.manual"),
                                Some(SessionState::PendingPr) => i18n.t("stage1.state.note.pending"),
                                Some(SessionState::StalledPr) => i18n.t("stage1.state.note.stalled"),
                                Some(SessionState::Conflict) => i18n.t("stage1.state.note.conflict"),
                            }
                        };

                        let onclick = {
                            let on_select_new_node = props.on_select_new_node.clone();
                            let on_select_edit_node = props.on_select_edit_node.clone();
                            let on_select_blocked = props.on_select_blocked.clone();
                            let node_name = node.name.clone();
                            let node_session = node_session.clone();
                            Callback::from(move |_| {
                                match node_session.as_ref().map(|session| &session.state) {
                                    None => {
                                        on_select_new_node.emit(node_name.clone());
                                    }
                                    Some(SessionState::Managed) | Some(SessionState::Manual) | Some(SessionState::StalledPr) => {
                                        let Some(session) = node_session.as_ref() else {
                                            on_select_blocked.emit(UiMessage::key("error.ui.session.missing_config"));
                                            return;
                                        };
                                        if session.spec.is_none() {
                                            on_select_blocked.emit(UiMessage::key("error.ui.session.missing_config"));
                                            return;
                                        }
                                        on_select_edit_node.emit((node_name.clone(), session.clone()));
                                    }
                                    Some(SessionState::PendingPr) => {
                                        on_select_blocked.emit(UiMessage::key("error.ui.operation.wait_inflight"));
                                    }
                                    Some(SessionState::Conflict) => {
                                        on_select_blocked.emit(UiMessage::key("error.ui.node.blocked_conflict"));
                                    }
                                }
                            })
                        };

                        html! {
                            <ShellButton
                                class={classes!(
                                    "autopeer-node-option",
                                    selected.then_some("is-selected"),
                                    (!selectable).then_some("is-unavailable")
                                )}
                                onclick={onclick}
                                disabled={props.loading || !selectable}
                            >
                                <span class="autopeer-node-option-head">
                                    <strong class="autopeer-node-name">{node.name.clone()}</strong>
                                    <span class="autopeer-node-option-status">
                                        <span class="autopeer-node-badge">{humanize_ip_support(i18n, &node.ip_support)}</span>
                                        <span class="autopeer-status-pill">{state_label}</span>
                                    </span>
                                </span>
                                <span class="autopeer-node-meta">{node_context_line(i18n, node)}</span>
                                if let Some(comment) = &node.comment {
                                    <span class="autopeer-node-note">{comment.clone()}</span>
                                }
                                if let Some(message) = node_session.as_ref().and_then(|session| session.message.as_ref()) {
                                    <span class="autopeer-node-note">{i18n.translate_message(message)}</span>
                                }
                                <span class="autopeer-node-state-note">{state_note}</span>
                            </ShellButton>
                        }
                    })}
                </div>
            }
            {render_error(i18n, &props.error)}
        </article>
    }
}
