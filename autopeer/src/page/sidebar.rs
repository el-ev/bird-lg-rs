use yew::prelude::*;

use crate::{
    i18n::I18n,
    models::{AuthSessionResponse, OperationState, OperationStatus, UiMessage},
};
use ui_components::shell::{ShellButton, ShellInput, ShellLine, ShellPrompt};

use super::{render_error, render_operation_progress};

#[derive(Properties, PartialEq)]
pub struct DashboardSidebarProps {
    pub i18n: I18n,
    pub loading: bool,
    pub auth_session: Option<AuthSessionResponse>,
    pub host_session: Option<AuthSessionResponse>,
    pub operation: Option<OperationStatus>,
    pub support_error: Option<UiMessage>,
    pub impersonate_asn: String,
    pub impersonate_mnt: String,
    pub on_impersonate_asn_change: Callback<String>,
    pub on_impersonate_mnt_change: Callback<String>,
    pub on_impersonate: Callback<MouseEvent>,
    pub on_return_to_host: Callback<MouseEvent>,
    pub on_retry_operation: Callback<String>,
    pub on_dismiss_operation: Callback<MouseEvent>,
    pub on_drop_operation: Callback<String>,
}

#[function_component(DashboardSidebar)]
pub fn dashboard_sidebar(props: &DashboardSidebarProps) -> Html {
    let i18n = &props.i18n;

    html! {
        <aside class="autopeer-sidebar">
            <article class="peering-card autopeer-panel autopeer-panel--compact">
                <div class="autopeer-panel-header">
                    <p class="autopeer-panel-kicker">{i18n.t("sidebar.your_session_kicker")}</p>
                    <h3 class="autopeer-panel-title">
                        {props.auth_session.as_ref().map(|session| format!("AS{}", session.asn)).unwrap_or_else(|| i18n.t("sidebar.no_active_session").to_string())}
                    </h3>
                    {props.auth_session.as_ref().map(|session| {
                        let auth_label = i18n.translate_message(&session.auth_method.label);
                        html! {
                            <p class="text-secondary">
                                {i18n.translate_params(
                                    "sidebar.session_authed_template",
                                    &[
                                        ("mnt", session.effective_mnt.as_str()),
                                        ("label", auth_label.as_str()),
                                    ],
                                )}
                            </p>
                        }
                    }).unwrap_or_default()}
                </div>
            </article>

            {props.host_session.as_ref().map(|host_session| {
                let auth_label = i18n.translate_message(&host_session.auth_method.label);
                html! {
                    <article class="peering-card autopeer-panel autopeer-panel--compact">
                        <div class="autopeer-panel-header">
                            <p class="autopeer-panel-kicker">{i18n.t("sidebar.support_kicker")}</p>
                            <h3 class="autopeer-panel-title">
                                {i18n.translate_params(
                                    "sidebar.host_asn_prefix",
                                    &[("asn", host_session.asn.as_str())],
                                )}
                            </h3>
                            <p class="text-secondary">
                                {i18n.translate_params(
                                    "sidebar.host_authed_template",
                                    &[
                                        ("mnt", host_session.effective_mnt.as_str()),
                                        ("label", auth_label.as_str()),
                                    ],
                                )}
                            </p>
                        </div>
                        <div class="autopeer-form-section autopeer-form-section--compact">
                            <ShellLine>
                                <ShellPrompt>{i18n.t("sidebar.impersonate_asn_label")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={props.impersonate_asn.clone()}
                                    on_change={props.on_impersonate_asn_change.clone()}
                                    placeholder={i18n.t("sidebar.impersonate_asn_placeholder")}
                                    disabled={props.loading}
                                />
                            </ShellLine>
                            <ShellLine>
                                <ShellPrompt>{i18n.t("sidebar.effective_mnt_label")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={props.impersonate_mnt.clone()}
                                    on_change={props.on_impersonate_mnt_change.clone()}
                                    placeholder={i18n.t("sidebar.impersonate_mnt_placeholder")}
                                    disabled={props.loading}
                                />
                            </ShellLine>
                            {render_error(i18n, &props.support_error)}
                            <div class="autopeer-inline-actions">
                                <ShellButton
                                    text={i18n.t("action.impersonate_this_asn")}
                                    onclick={props.on_impersonate.clone()}
                                    disabled={props.loading || props.impersonate_asn.trim().is_empty()}
                                />
                                if props.auth_session.as_ref().map(|session| session.asn.as_str()) != Some(host_session.asn.as_str()) {
                                    <ShellButton
                                        text={i18n.t("action.return_to_host_asn")}
                                        onclick={props.on_return_to_host.clone()}
                                        disabled={props.loading}
                                    />
                                }
                            </div>
                        </div>
                    </article>
                }
            }).unwrap_or_default()}

            if let Some(operation_status) = &props.operation {
                <article class="peering-card autopeer-panel autopeer-panel--compact autopeer-status-card">
                    <div class="autopeer-panel-header">
                        <p class="autopeer-panel-kicker">{i18n.t("sidebar.current_operation")}</p>
                        <h3 class="autopeer-panel-title">
                            {format!("{} {}", i18n.t(operation_status.kind.i18n_key()), operation_status.node)}
                        </h3>
                        <span class="autopeer-status-pill">{i18n.t(operation_status.state.i18n_key())}</span>
                        if operation_status.failure_details.is_none() {
                            if let Some(message) = &operation_status.message {
                                <p class="text-secondary">{i18n.translate_message(message)}</p>
                            }
                        }
                    </div>
                    {render_operation_progress(i18n, operation_status)}
                    if let Some(details) = &operation_status.failure_details {
                        <div class="autopeer-failure-details">
                            <p class="autopeer-failure-stage">
                                <strong>{i18n.t("operation.failure.stage")}{": "}</strong>
                                {i18n.t(details.stage.i18n_key())}
                                if let Some(step) = &details.step {
                                    {format!(" — {}", step)}
                                }
                            </p>
                            if let Some(conclusion) = &details.conclusion {
                                <p class="text-secondary">
                                    <strong>{i18n.t("operation.failure.conclusion")}{": "}</strong>
                                    {conclusion}
                                </p>
                            }
                            if let Some(annotation) = &details.annotation {
                                <pre class="autopeer-failure-annotation">{annotation}</pre>
                            }
                        </div>
                    }
                    <div class="autopeer-links">
                        if let Some(pr_url) = &operation_status.pull_request_url {
                            <a href={pr_url.clone()} target="_blank" rel="noreferrer">{i18n.t("action.open_pr")}</a>
                        }
                        if let Some(run_url) = &operation_status.workflow_run_url {
                            <a href={run_url.clone()} target="_blank" rel="noreferrer">{i18n.t("action.workflow_run")}</a>
                        }
                        if operation_status.state.is_terminal() {
                            if operation_status.state == OperationState::Failed && operation_status.pull_request_url.is_some() {
                                <button
                                    class="autopeer-link-button"
                                    onclick={props.on_retry_operation.reform({
                                        let id = operation_status.id.clone();
                                        move |_: MouseEvent| id.clone()
                                    })}
                                    disabled={props.loading}
                                >
                                    {i18n.t("action.retry")}
                                </button>
                            }
                            <button
                                class="autopeer-link-button autopeer-link-button--muted"
                                onclick={props.on_dismiss_operation.clone()}
                            >
                                {i18n.t("action.dismiss_operation")}
                            </button>
                        }
                    </div>
                </article>
            }
        </aside>
    }
}
