mod manage_callbacks;
mod review;
mod select_node;
mod session_details;
mod sidebar;
mod verify_method;

use manage_callbacks::build_manage_callbacks;
use review::ReviewPanel;
use select_node::SelectNodePanel;
use session_details::SessionDetailsPanel;
use sidebar::DashboardSidebar;
use ui_components::shell::{ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect};
use verify_method::VerifyMethodPanel;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::{
    controller::{AutoPeerController, OngoingTask, use_autopeer_controller},
    i18n::{I18n, Locale, use_i18n},
    models::{NodeView, OperationFailureStage, OperationState, OperationStatus, UiMessage},
    store::{AutoPeerStep, PeerConfigStage, SessionDraft, SessionDraftField},
    update_form::{
        Peer6AddressKind, SessionDraftTouchedControls, detect_peer6_address_kind,
        displayed_node_ipv4_visibility, displayed_peer6_address_kind,
        session_details_live_validation, session_details_submission_error,
        should_display_node_ipv4,
    },
};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn render_readonly_block(label: &str, content: String) -> Html {
    let rows = content.lines().count().max(1);
    let on_select_all = Callback::from(move |event: MouseEvent| {
        let target: HtmlTextAreaElement = event.target_unchecked_into();
        target.select();
    });
    let on_focus = Callback::from(move |event: FocusEvent| {
        let target: HtmlTextAreaElement = event.target_unchecked_into();
        target.select();
    });

    html! {
        <div class="autopeer-command-group">
            <div class="autopeer-command-label">{label}</div>
            <div class="autopeer-command-block">
                <textarea
                    class="autopeer-command-textarea"
                    readonly=true
                    spellcheck="false"
                    rows={rows.to_string()}
                    value={content}
                    onclick={on_select_all}
                    onfocus={on_focus}
                />
            </div>
        </div>
    }
}

fn live_validation_message(i18n: &I18n, message: Option<&str>) -> Html {
    match message {
        Some(message) => html! {
            <p class="autopeer-live-validation" aria-live="polite">{i18n.translate_owned(message)}</p>
        },
        None => Html::default(),
    }
}

fn help_hint(i18n: &I18n, key: &'static str) -> Html {
    html! {
        <span class="shell-help-hint" data-hint={i18n.t(key)}>{"[?]"}</span>
    }
}

fn live_validation_messages(i18n: &I18n, messages: &[String]) -> Html {
    if messages.is_empty() {
        return Html::default();
    }

    html! {
        <div class="autopeer-live-validation-list" aria-live="polite">
            {for messages.iter().map(|message| html! {
                <p class="autopeer-live-validation">{i18n.translate_owned(message)}</p>
            })}
        </div>
    }
}

fn live_validation_block(i18n: &I18n, message: Option<&str>, content: Html) -> Html {
    html! {
        <div class={classes!(
            "autopeer-validation-block",
            message.is_some().then_some("autopeer-validation-block--invalid")
        )}>
            {content}
            {live_validation_message(i18n, message)}
        </div>
    }
}

fn live_validation_block_multi(i18n: &I18n, messages: &[String], content: Html) -> Html {
    html! {
        <div class={classes!(
            "autopeer-validation-block",
            (!messages.is_empty()).then_some("autopeer-validation-block--invalid")
        )}>
            {content}
            {live_validation_messages(i18n, messages)}
        </div>
    }
}

fn generate_wg_psk() -> Option<String> {
    let window = web_sys::window()?;
    let crypto = window.crypto().ok()?;
    let buf = js_sys::Uint8Array::new_with_length(32);
    crypto.get_random_values_with_array_buffer_view(&buf).ok()?;
    let bytes: Vec<u8> = buf.to_vec();
    let binary: String = bytes.iter().map(|&b| char::from(b)).collect();
    window.btoa(&binary).ok()
}

fn ssh_sign_command(challenge_text: &str) -> String {
    format!("ssh-keygen -Y sign -f <PRIVATE_KEY_PATH> -n file <<'EOF'\n{challenge_text}\nEOF")
}

fn pgp_export_command(key_id: &str) -> String {
    if key_id.trim().is_empty() {
        "gpg --armor --export <KEYID_OR_FINGERPRINT>".to_string()
    } else {
        format!("gpg --armor --export {key_id}")
    }
}

fn pgp_sign_command(challenge_text: &str, key_id: &str) -> String {
    if key_id.trim().is_empty() {
        format!("gpg --armor --clearsign <<'EOF'\n{challenge_text}\nEOF")
    } else {
        format!("gpg --armor --local-user {key_id} --clearsign <<'EOF'\n{challenge_text}\nEOF")
    }
}

fn render_error(i18n: &I18n, error: &Option<UiMessage>) -> Html {
    match error {
        Some(message) => html! {
            <ShellLine>
                <span class="error-message">{i18n.translate_message(message)}</span>
            </ShellLine>
        },
        None => Html::default(),
    }
}

fn render_ongoing_tasks(i18n: &I18n, tasks: &[OngoingTask]) -> Html {
    if tasks.is_empty() {
        return Html::default();
    }
    html! {
        <div class="autopeer-ongoing-tasks">
            {for tasks.iter().map(|task| html! {
                <div key={task.id.to_string()} class="autopeer-ongoing-task">
                    <ShellLine>
                        <span class="text-secondary">
                            {i18n.translate_message(&task.message)}
                        </span>
                    </ShellLine>
                </div>
            })}
        </div>
    }
}

fn humanize_token(i18n: &I18n, token: &str) -> String {
    match token {
        "N" => i18n.t("location.direction.n").to_string(),
        "S" => i18n.t("location.direction.s").to_string(),
        "E" => i18n.t("location.direction.e").to_string(),
        "W" => i18n.t("location.direction.w").to_string(),
        "NE" => i18n.t("location.direction.ne").to_string(),
        "NW" => i18n.t("location.direction.nw").to_string(),
        "SE" => i18n.t("location.direction.se").to_string(),
        "SW" => i18n.t("location.direction.sw").to_string(),
        other if other.len() <= 3 && other.chars().all(|char| char.is_ascii_uppercase()) => {
            other.to_string()
        }
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        }
    }
}

fn humanize_region(i18n: &I18n, region: &Option<String>) -> Option<String> {
    region.as_ref().map(|value| {
        let key = format!("location.region.{}", value.to_lowercase());
        let translated = i18n.translate_owned(&key);
        if translated != key {
            return translated;
        }
        value
            .split('_')
            .map(|token| humanize_token(i18n, token))
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn humanize_ip_support(i18n: &I18n, value: &str) -> &'static str {
    match value {
        "ipv4" => i18n.t("node.transport.ipv4"),
        "ipv6" => i18n.t("node.transport.ipv6"),
        "dual" => i18n.t("node.transport.dual_stack"),
        _ => i18n.t("node.transport.dual_stack"),
    }
}

fn node_context_line(i18n: &I18n, node: &NodeView) -> String {
    let mut parts = Vec::new();

    if let Some(region) = humanize_region(i18n, &node.region) {
        parts.push(region);
    }
    if let Some(country) = &node.country {
        let key = format!("location.country.{}", country.to_lowercase());
        let translated = i18n.translate_owned(&key);
        if translated != key {
            parts.push(translated);
        } else {
            parts.push(country.to_string());
        }
    }

    if parts.is_empty() {
        humanize_ip_support(i18n, &node.ip_support).to_string()
    } else {
        parts.join(", ")
    }
}

fn review_item(label: &'static str, value: String, changed: bool) -> Html {
    html! {
        <div class={classes!("autopeer-review-item", changed.then_some("autopeer-review-item--changed"))}>
            <span class="autopeer-review-label">{label}</span>
            <strong class="autopeer-review-value">{value}</strong>
        </div>
    }
}

fn optional_review_item(label: &'static str, value: &str, changed: bool) -> Html {
    match value.trim() {
        "" => Html::default(),
        value => review_item(label, value.to_string(), changed),
    }
}

fn render_peering_field(label: &'static str, value: Option<&str>, active: bool) -> Html {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => html! {
            <>
                <dt class={classes!("peering-label", active.then_some("peering-label--active"))}>{label}</dt>
                <dd class="peering-value">{value}</dd>
            </>
        },
        None => Html::default(),
    }
}

fn autopeer_node_endpoint_port(asn: &str) -> String {
    let suffix = asn
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("2{suffix}")
}

fn render_inventory_peering_review(
    i18n: &I18n,
    node: Option<&NodeView>,
    active_asn: &str,
    draft: &SessionDraft,
    looking_glass_site_href: &str,
) -> Html {
    let Some(node) = node else {
        return Html::default();
    };
    let Some(peering) = node.peering.as_ref() else {
        return Html::default();
    };
    let node_endpoint = node
        .endpoint_host
        .as_ref()
        .map(|host| format!("{host}:{}", autopeer_node_endpoint_port(active_asn)));
    if node_endpoint.is_none()
        && peering.ipv4.is_none()
        && peering.ipv6.is_none()
        && peering.link_local_ipv6.is_none()
        && peering.wg_pubkey.is_none()
        && peering.endpoint.is_none()
        && peering.comment.is_none()
    {
        return Html::default();
    }

    html! {
        <div class="autopeer-review-section">
            <p class="autopeer-review-section-title">{i18n.t("stage3.review.our_node_details")}</p>
            <dl class="peering-grid autopeer-review-peering-grid">
                {render_peering_field(i18n.t("stage3.review.our_endpoint"), node_endpoint.as_deref(), true)}
                {render_peering_field(i18n.t("stage3.review.our_ipv4"), peering.ipv4.as_deref(), draft.peer4_is_active())}
                {render_peering_field(i18n.t("stage3.review.our_ipv6"), peering.ipv6.as_deref(),
                    draft.peer6_is_active() && detect_peer6_address_kind(&draft.peer6) == Some(Peer6AddressKind::Ula))}
                {render_peering_field(i18n.t("stage3.review.our_link_local_ipv6"), peering.link_local_ipv6.as_deref(),
                    draft.peer6_is_active() && detect_peer6_address_kind(&draft.peer6) == Some(Peer6AddressKind::LinkLocal))}
                {render_peering_field(i18n.t("stage3.review.our_wg_pubkey"), peering.wg_pubkey.as_deref(), true)}
                {render_peering_field(i18n.t("stage3.review.our_node_note"), peering.comment.as_deref(), false)}
                if !looking_glass_site_href.is_empty() {
                    <dt class="peering-label">{i18n.t("stage3.review.check_session_label")}</dt>
                    <dd class="peering-value">
                        <a
                            href={format!(
                                "{}/node/{}/protocol/dn42_{}",
                                looking_glass_site_href.trim_end_matches('/'),
                                node.name,
                                &active_asn[active_asn.len().saturating_sub(4)..],
                            )}
                            target="_blank"
                            rel="noreferrer"
                        >
                            {format!("dn42_{}", &active_asn[active_asn.len().saturating_sub(4)..])}
                        </a>
                    </dd>
                }
            </dl>
        </div>
    }
}

fn render_flow_steps(
    i18n: &I18n,
    stage: PeerConfigStage,
    on_step_click: &Callback<PeerConfigStage>,
) -> Html {
    let steps = [
        PeerConfigStage::SelectNode,
        PeerConfigStage::SessionDetails,
        PeerConfigStage::Review,
    ];

    html! {
        <ol class="autopeer-flow-steps">
            {for steps.into_iter().map(|candidate| {
                let is_complete = candidate.index() < stage.index();
                let state_class = if candidate == stage {
                    "is-active"
                } else if is_complete {
                    "is-complete"
                } else {
                    "is-upcoming"
                };
                let onclick = if is_complete {
                    let cb = on_step_click.clone();
                    Some(Callback::from(move |_: MouseEvent| cb.emit(candidate)))
                } else {
                    None
                };
                html! {
                    <li class={classes!("autopeer-flow-step", state_class)} {onclick}>
                        <span class="autopeer-flow-step-index">{candidate.index() + 1}</span>
                        <span class="autopeer-flow-step-copy">
                            <strong>{i18n.t(candidate.title_key())}</strong>
                            <span>{i18n.t(candidate.description_key())}</span>
                        </span>
                    </li>
                }
            })}
        </ol>
    }
}

fn render_operation_progress(i18n: &I18n, operation: &OperationStatus) -> Html {
    let labels = [
        i18n.t("operation.progress.branch"),
        i18n.t("operation.progress.checks"),
        i18n.t("operation.progress.apply"),
        i18n.t("operation.progress.merge"),
        i18n.t("operation.progress.done"),
    ];
    let active_index = match operation.state {
        OperationState::PendingPullRequest => 0,
        OperationState::PendingChecks => 1,
        OperationState::Applying => 2,
        OperationState::PendingMerge => 3,
        OperationState::Completed => 4,
        OperationState::Failed | OperationState::Conflict => operation
            .failure_details
            .as_ref()
            .map(|details| match details.stage {
                OperationFailureStage::Checks => 1,
                OperationFailureStage::Preflight | OperationFailureStage::Apply => 2,
                OperationFailureStage::Merge => 3,
            })
            .unwrap_or(2),
    };
    let failed = matches!(
        operation.state,
        OperationState::Failed | OperationState::Conflict
    );

    html! {
        <ol class="autopeer-progress">
            {for labels.iter().enumerate().map(|(index, label)| {
                let class = if failed && index == active_index {
                    "is-failed"
                } else if index < active_index {
                    "is-complete"
                } else if index == active_index {
                    "is-current"
                } else {
                    "is-upcoming"
                };
                html! {
                    <li class={classes!("autopeer-progress-step", class)}>
                        <span>{*label}</span>
                    </li>
                }
            })}
        </ol>
    }
}

// ---------------------------------------------------------------------------
// Main page component
// ---------------------------------------------------------------------------

#[function_component(AutoPeerPage)]
pub fn auto_peer_page() -> Html {
    let i18n = use_i18n();
    let default_autopeer_home_href = String::from("/");
    let default_looking_glass_href = String::new();
    let AutoPeerController {
        autopeer_site_href,
        looking_glass_site_href,
        oidc_methods,
        step,
        asn,
        challenge_text,
        methods,
        selected_method,
        auth_session,
        host_session,
        nodes,
        sessions,
        draft,
        touched_fields,
        editing_node,
        config_stage,
        retire_confirmation,
        delete_confirmation,
        operation,
        error,
        support_error,
        ongoing_tasks,
        impersonate_asn,
        impersonate_mnt,
        ssh_signature,
        selected_pgp_key,
        pgp_public_key,
        pgp_signed_message,
        pgp_key_lookups,
        selected_email_maintainer,
        registry_email_code,
        registry_email_sent_to,
        on_asn_change,
        on_submit_asn,
        on_asn_keydown,
        on_enter_oidc,
        on_select_method,
        on_select_method_back,
        on_verify_back,
        on_verify,
        on_selected_email_maintainer_change,
        on_registry_email_code_change,
        on_send_registry_email,
        on_refresh,
        on_logout,
        on_impersonate_asn_change,
        on_impersonate_mnt_change,
        on_impersonate,
        on_return_to_host,
        on_submit_session,
        on_retire_selected_session,
        on_delete_selected_session,
        on_retry_operation,
        on_dismiss_operation,
        on_drop_operation,
    } = use_autopeer_controller(default_autopeer_home_href, default_looking_glass_href);
    let loading = !ongoing_tasks.is_empty();
    let focused_field = use_state(|| None::<SessionDraftField>);
    let committed_peer4_visibility = use_state(|| should_display_node_ipv4(&draft));
    let committed_peer6_kind = use_state(|| detect_peer6_address_kind(draft.peer6.as_str()));
    let committed_tunnel_message = use_mut_ref(|| None::<String>);
    let committed_peer6_messages = use_mut_ref(Vec::<String>::new);
    let psk_copied = use_state(|| false);
    let locale_value = i18n.locale().code();

    {
        let committed_peer4_visibility = committed_peer4_visibility.clone();
        let peer4_value = draft.peer4.clone();
        let focused_field_value = *focused_field;
        let draft_value = (*draft).clone();
        use_effect_with(
            (peer4_value, focused_field_value),
            move |(_, focused_field_value)| {
                if *focused_field_value != Some(SessionDraftField::Peer4) {
                    committed_peer4_visibility.set(should_display_node_ipv4(&draft_value));
                }
                || ()
            },
        );
    }

    {
        let committed_peer6_kind = committed_peer6_kind.clone();
        let peer6_value = draft.peer6.clone();
        let focused_field_value = *focused_field;
        use_effect_with(
            (peer6_value, focused_field_value),
            move |(peer6_value, focused_field_value)| {
                if *focused_field_value != Some(SessionDraftField::Peer6) {
                    committed_peer6_kind.set(detect_peer6_address_kind(peer6_value));
                }
                || ()
            },
        );
    }

    let on_locale_change = {
        let i18n = i18n.clone();
        Callback::from(move |event: Event| {
            let target: web_sys::HtmlSelectElement = event.target_unchecked_into();
            if let Some(locale) = Locale::from_code(&target.value()) {
                i18n.set_locale(locale);
            }
        })
    };

    let content = match &*step {
        AutoPeerStep::LoadingConfig => html! {
            <div class="autopeer-step">
                <ShellLine>
                    <ShellPrompt>{i18n.t("prompt.autopeer")}</ShellPrompt>
                    {" "}{i18n.t("step.loading_config.prompt")}
                </ShellLine>
                <div class="autopeer-ongoing-task">
                    <ShellLine>
                        <span class="text-secondary">{i18n.t("step.loading_config.message")}</span>
                    </ShellLine>
                </div>
                {render_error(&i18n, &error)}
            </div>
        },
        AutoPeerStep::EnterAsn => html! {
            <div class="autopeer-step">
                <ShellLine>
                    <ShellPrompt>{i18n.t("prompt.autopeer")}</ShellPrompt>
                    {" "}{i18n.t("step.enter_asn.prompt")}
                </ShellLine>
                <ShellLine>
                    <ShellPrompt>{i18n.t("prompt.asn")}</ShellPrompt>
                    {" "}
                    <ShellInput
                        value={(*asn).clone()}
                        on_change={on_asn_change}
                        placeholder={i18n.t("step.enter_asn.placeholder")}
                        disabled={loading}
                        on_keydown={on_asn_keydown}
                    />
                </ShellLine>
                {render_ongoing_tasks(&i18n, ongoing_tasks.tasks())}
                {render_error(&i18n, &error)}
                <ShellLine>
                    <ShellButton
                        text={i18n.t("action.find_registry_auth")}
                        onclick={on_submit_asn}
                        disabled={loading || asn.trim().is_empty()}
                    />
                </ShellLine>
                if !oidc_methods.is_empty() {
                    <div class="autopeer-entry-alt">
                        <div class="autopeer-entry-alt-copy">
                            {i18n.t("step.enter_asn.oidc_alt")}
                        </div>
                        <div class="autopeer-challenge-list">
                            {for oidc_methods.iter().map(|method| {
                                let on_enter_oidc = on_enter_oidc.clone();
                                let method = method.clone();
                                let method_copy = method.clone();
                                let method_label = i18n.translate_message(&method.label);
                                let method_description = i18n.translate_message(&method.description);
                                let onclick = Callback::from(move |_| {
                                    on_enter_oidc.emit(method_copy.clone());
                                });

                                html! {
                                    <ShellLine>
                                        <ShellButton
                                            text={i18n.translate_params(
                                                "step.enter_asn.continue_with",
                                                &[("provider", method_label.as_str())],
                                            )}
                                            onclick={onclick}
                                            disabled={loading}
                                        />
                                        <span class="autopeer-method-desc">
                                            {" - "}{method_description}
                                        </span>
                                    </ShellLine>
                                }
                            })}
                        </div>
                    </div>
                }
            </div>
        },
        AutoPeerStep::SelectMethod => {
            html! {
                <div class="autopeer-step">
                    <ShellLine>
                        <ShellPrompt>{i18n.t("prompt.autopeer")}</ShellPrompt>
                        {" "}{i18n.translate_params(
                            "step.select_method.found_for_as",
                            &[("asn", asn.as_str())],
                        )}
                    </ShellLine>
                    <div class="autopeer-challenge-list">
                        {for methods.iter().map(|method| {
                            let on_select_method = on_select_method.clone();
                            let method_value = method.clone();
                            let method_description = i18n.translate_message(&method.description);
                            let onclick = Callback::from(move |_| {
                                on_select_method.emit(method_value.clone());
                            });

                            html! {
                                <ShellLine>
                                    <ShellButton
                                        text={i18n.translate_message(&method.label)}
                                        onclick={onclick}
                                        disabled={loading}
                                    />
                                    <span class="autopeer-method-desc">
                                        {" - "}{method_description}
                                    </span>
                                </ShellLine>
                            }
                        })}
                    </div>
                    {render_ongoing_tasks(&i18n, ongoing_tasks.tasks())}
                    {render_error(&i18n, &error)}
                    <ShellLine>
                        <ShellButton
                            text={i18n.t("action.back")}
                            onclick={on_select_method_back.clone()}
                            disabled={loading}
                        />
                    </ShellLine>
                </div>
            }
        }
        AutoPeerStep::VerifyMethod => {
            html! {
                <VerifyMethodPanel
                    i18n={i18n.clone()}
                    loading={loading}
                    asn={(*asn).clone()}
                    selected_method={(*selected_method).clone()}
                    challenge_text={(*challenge_text).clone()}
                    ssh_signature={(*ssh_signature).clone()}
                    on_ssh_signature_change={Callback::from({
                        let ssh_signature = ssh_signature.clone();
                        move |v| ssh_signature.set(v)
                    })}
                    selected_pgp_key={(*selected_pgp_key).clone()}
                    pgp_public_key={(*pgp_public_key).clone()}
                    pgp_signed_message={(*pgp_signed_message).clone()}
                    pgp_key_lookups={(*pgp_key_lookups).clone()}
                    on_pgp_key_change={Callback::from({
                        let selected_pgp_key = selected_pgp_key.clone();
                        move |v| selected_pgp_key.set(v)
                    })}
                    on_pgp_public_key_change={Callback::from({
                        let pgp_public_key = pgp_public_key.clone();
                        move |v| pgp_public_key.set(v)
                    })}
                    on_pgp_signed_message_change={Callback::from({
                        let pgp_signed_message = pgp_signed_message.clone();
                        move |v| pgp_signed_message.set(v)
                    })}
                    selected_email_maintainer={(*selected_email_maintainer).clone()}
                    registry_email_code={(*registry_email_code).clone()}
                    registry_email_sent_to={(*registry_email_sent_to).clone()}
                    on_email_maintainer_change={on_selected_email_maintainer_change}
                    on_registry_email_code_change={on_registry_email_code_change}
                    on_send_registry_email={on_send_registry_email}
                    ongoing_tasks={ongoing_tasks.tasks().to_vec()}
                    error={(*error).clone()}
                    on_verify={on_verify}
                    on_verify_back={on_verify_back}
                />
            }
        }
        AutoPeerStep::ManageSessions => {
            let auth_summary = (*auth_session).clone();
            let host_summary = (*host_session).clone();
            let host_session_active = auth_summary
                .as_ref()
                .zip(host_summary.as_ref())
                .map(|(active, host)| active.asn == host.asn)
                .unwrap_or(false);
            let editing_node_value = (*editing_node).clone();
            let active_stage =
                if editing_node_value.is_some() && *config_stage == PeerConfigStage::SelectNode {
                    PeerConfigStage::SessionDetails
                } else {
                    *config_stage
                };
            let selected_node_name = editing_node_value.as_deref().or_else(|| {
                let selected = draft.node.trim();
                (!selected.is_empty()).then_some(selected)
            });
            let selected_node = selected_node_name
                .and_then(|name| nodes.iter().find(|node| node.name == name).cloned());
            let selected_session =
                selected_node_name.and_then(|name| sessions.iter().find(|s| s.node == name));
            let retire_confirmation_value = *retire_confirmation;
            let delete_confirmation_value = *delete_confirmation;
            let active_asn = auth_summary
                .as_ref()
                .map(|session| session.asn.clone())
                .unwrap_or_else(|| (*asn).clone());
            let node_inventory_ipv4 = selected_node
                .as_ref()
                .and_then(|node| node.peering.as_ref())
                .and_then(|peering| peering.ipv4.clone());
            let node_inventory_ipv6 = selected_node
                .as_ref()
                .and_then(|node| node.peering.as_ref())
                .and_then(|peering| peering.ipv6.clone());
            let node_inventory_link_local_ipv6 = selected_node
                .as_ref()
                .and_then(|node| node.peering.as_ref())
                .and_then(|peering| peering.link_local_ipv6.clone());
            let draft_error =
                session_details_submission_error(&draft, node_inventory_link_local_ipv6.as_deref());
            let draft_is_valid = draft_error.is_none();
            let live_validation = {
                let mut lv = session_details_live_validation(
                    &draft,
                    &touched_fields,
                    *focused_field,
                    node_inventory_link_local_ipv6.as_deref(),
                );
                let tunnel_field_focused = matches!(
                    *focused_field,
                    Some(
                        SessionDraftField::Peer4
                            | SessionDraftField::Peer6
                            | SessionDraftField::Own6
                    )
                );
                if tunnel_field_focused {
                    lv.tunnel_message = committed_tunnel_message.borrow().clone();
                } else {
                    *committed_tunnel_message.borrow_mut() = lv.tunnel_message.clone();
                }
                if *focused_field == Some(SessionDraftField::Peer6) {
                    lv.peer6_messages = committed_peer6_messages.borrow().clone();
                } else {
                    *committed_peer6_messages.borrow_mut() = lv.peer6_messages.clone();
                }
                lv
            };
            let show_node_ipv4 =
                displayed_node_ipv4_visibility(&draft, *focused_field, *committed_peer4_visibility);
            let peer6_kind =
                displayed_peer6_address_kind(&draft.peer6, *focused_field, *committed_peer6_kind);
            let own6_placeholder = match peer6_kind {
                Some(Peer6AddressKind::LinkLocal) => {
                    node_inventory_link_local_ipv6.clone().unwrap_or_else(|| {
                        i18n.t("stage2.field.own6_link_local.placeholder")
                            .to_string()
                    })
                }
                _ => i18n
                    .t("stage2.field.own6_link_local.placeholder")
                    .to_string(),
            };

            let manage_cbs = build_manage_callbacks(
                &draft,
                &touched_fields,
                &editing_node,
                &config_stage,
                &focused_field,
                &committed_peer6_kind,
                &sessions,
                &nodes,
                &error,
                &psk_copied,
                node_inventory_link_local_ipv6,
            );

            let on_select_new_node = {
                let editing_node = editing_node.clone();
                let draft = draft.clone();
                let config_stage = config_stage.clone();
                let touched_fields = touched_fields.clone();
                let error = error.clone();
                Callback::from(move |node_name: String| {
                    error.set(None);
                    editing_node.set(None);
                    draft.set(SessionDraft {
                        node: node_name,
                        ..SessionDraft::default()
                    });
                    touched_fields.set(SessionDraftTouchedControls::new());
                    config_stage.set(PeerConfigStage::SessionDetails);
                })
            };

            let on_select_edit_node = {
                let editing_node = editing_node.clone();
                let draft = draft.clone();
                let config_stage = config_stage.clone();
                let touched_fields = touched_fields.clone();
                let error = error.clone();
                Callback::from(
                    move |(node_name, session): (String, crate::models::SessionView)| {
                        error.set(None);
                        editing_node.set(Some(node_name.clone()));
                        draft.set(SessionDraft::from_session_view(&node_name, &session));
                        touched_fields.set(SessionDraftTouchedControls::new());
                        config_stage.set(PeerConfigStage::SessionDetails);
                    },
                )
            };

            let on_select_blocked = {
                let error = error.clone();
                Callback::from(move |msg: UiMessage| error.set(Some(msg)))
            };

            let main_panel = match active_stage {
                PeerConfigStage::SelectNode => html! {
                    <SelectNodePanel
                        i18n={i18n.clone()}
                        loading={loading}
                        nodes={(*nodes).clone()}
                        sessions={(*sessions).clone()}
                        selected_node_name={selected_node_name.map(str::to_string)}
                        error={(*error).clone()}
                        on_select_new_node={on_select_new_node}
                        on_select_edit_node={on_select_edit_node}
                        on_select_blocked={on_select_blocked}
                    />
                },
                PeerConfigStage::SessionDetails => html! {
                    <SessionDetailsPanel
                        i18n={i18n.clone()}
                        loading={loading}
                        draft={(*draft).clone()}
                        editing_node={editing_node_value.clone()}
                        selected_node={selected_node.clone()}
                        selected_session={selected_session.cloned()}
                        live_validation={live_validation}
                        touched_fields={(*touched_fields).clone()}
                        focused_field={*focused_field}
                        show_node_ipv4={show_node_ipv4}
                        peer6_kind={peer6_kind}
                        own6_placeholder={own6_placeholder}
                        node_inventory_ipv4={node_inventory_ipv4.clone()}
                        node_inventory_ipv6={node_inventory_ipv6}
                        psk_copied={*psk_copied}
                        retire_confirmation={retire_confirmation_value}
                        delete_confirmation={delete_confirmation_value}
                        draft_is_valid={draft_is_valid}
                        ongoing_tasks={ongoing_tasks.tasks().to_vec()}
                        error={(*error).clone()}
                        on_cancel_edit={manage_cbs.on_cancel_edit.clone()}
                        on_field_focus={manage_cbs.on_field_focus}
                        on_field_blur={manage_cbs.on_field_blur}
                        on_peer6_blur={manage_cbs.on_peer6_blur}
                        on_text_field_change={manage_cbs.on_text_field_change}
                        on_comment_change={manage_cbs.on_comment_change}
                        on_peer6_change={manage_cbs.on_peer6_change}
                        on_toggle_ipv4={manage_cbs.on_toggle_ipv4}
                        on_toggle_ipv6={manage_cbs.on_toggle_ipv6}
                        on_toggle_mp_bgp={manage_cbs.on_toggle_mp_bgp}
                        on_toggle_extended_next_hop={manage_cbs.on_toggle_extended_next_hop}
                        on_change_mp_bgp_transport={manage_cbs.on_change_mp_bgp_transport}
                        on_change_peering_strategy={manage_cbs.on_change_peering_strategy}
                        on_toggle_encrypt_endpoint={manage_cbs.on_toggle_encrypt_endpoint}
                        on_psk_action={manage_cbs.on_psk_action}
                        on_change_node={manage_cbs.on_change_node.clone()}
                        on_continue_to_review={manage_cbs.on_continue_to_review}
                        on_retire_selected_session={on_retire_selected_session.clone()}
                        on_delete_selected_session={on_delete_selected_session.clone()}
                        on_retry_operation={on_retry_operation.clone()}
                        on_drop_operation={on_drop_operation.clone()}
                    />
                },
                PeerConfigStage::Review => html! {
                    <ReviewPanel
                        i18n={i18n.clone()}
                        loading={loading}
                        draft={(*draft).clone()}
                        original_draft={editing_node_value.as_ref().and_then(|_|
                            selected_session.map(|s| SessionDraft::from_session_view(&s.node, s))
                        )}
                        editing_node={editing_node_value.clone()}
                        selected_node={selected_node.clone()}
                        active_asn={active_asn}
                        looking_glass_site_href={(*looking_glass_site_href).clone()}
                        draft_is_valid={draft_is_valid}
                        ongoing_tasks={ongoing_tasks.tasks().to_vec()}
                        error={(*error).clone()}
                        on_back_to_details={manage_cbs.on_back_to_details}
                        on_cancel_edit={manage_cbs.on_cancel_edit}
                        on_change_node={manage_cbs.on_change_node}
                        on_submit_session={on_submit_session}
                    />
                },
            };

            html! {
                <div class="autopeer-dashboard">
                    <section class="autopeer-overview peering-card">
                        <div>
                            <p class="autopeer-panel-kicker">{i18n.t("dashboard.flow_kicker")}</p>
                            <h3 class="autopeer-panel-title">
                                {if host_session_active {
                                    i18n.t("dashboard.host_readonly_title")
                                } else if editing_node_value.is_some() {
                                    i18n.t("dashboard.update_managed_title")
                                } else {
                                    i18n.t("dashboard.create_or_manage_title")
                                }}
                            </h3>
                            <p class="text-secondary">
                                {if host_session_active {
                                    i18n.t("dashboard.host_readonly_body")
                                } else {
                                    i18n.t("dashboard.create_or_manage_body")
                                }}
                            </p>
                        </div>
                        <div class="autopeer-overview-meta">
                            <div class="autopeer-overview-summary">
                                {auth_summary.as_ref().map(|session| {
                                    let auth_label = i18n.translate_message(&session.auth_method.label);
                                    html! {
                                        <>
                                            <span class="autopeer-status-pill">{format!("AS{}", session.asn)}</span>
                                            <span class="autopeer-node-badge">
                                                {i18n.translate_params(
                                                    "dashboard.session_badge_template",
                                                    &[
                                                        ("mnt", session.effective_mnt.as_str()),
                                                        ("label", auth_label.as_str()),
                                                    ],
                                                )}
                                            </span>
                                        </>
                                    }
                                }).unwrap_or_default()}
                            </div>
                            <div class="autopeer-overview-actions">
                                <ShellButton text={i18n.t("action.refresh")} onclick={on_refresh.clone()} disabled={loading} />
                                <ShellButton text={i18n.t("action.logout")} onclick={on_logout} disabled={loading} />
                            </div>
                        </div>
                    </section>

                    <div class="autopeer-workspace">
                        <div class="autopeer-main">
                            if host_session_active {
                                <article class="peering-card autopeer-panel">
                                    <div class="autopeer-panel-header">
                                        <p class="autopeer-panel-kicker">{i18n.t("sidebar.support_kicker")}</p>
                                        <h3 class="autopeer-panel-title">{i18n.t("sidebar.support_mode_title")}</h3>
                                        <p class="text-secondary">
                                            {i18n.t("sidebar.support_mode_body")}
                                        </p>
                                    </div>
                                </article>
                            } else {
                                <>
                                    {render_flow_steps(&i18n, active_stage, &manage_cbs.on_step_click)}
                                    {main_panel}
                                </>
                            }
                        </div>

                        <DashboardSidebar
                            i18n={i18n.clone()}
                            loading={loading}
                            auth_session={auth_summary}
                            host_session={host_summary}
                            operation={(*operation).clone()}
                            looking_glass_site_href={(*looking_glass_site_href).clone()}
                            support_error={(*support_error).clone()}
                            impersonate_asn={(*impersonate_asn).clone()}
                            impersonate_mnt={(*impersonate_mnt).clone()}
                            on_impersonate_asn_change={on_impersonate_asn_change}
                            on_impersonate_mnt_change={on_impersonate_mnt_change}
                            on_impersonate={on_impersonate}
                            on_return_to_host={on_return_to_host}
                            on_retry_operation={on_retry_operation}
                            on_dismiss_operation={on_dismiss_operation}
                            on_drop_operation={on_drop_operation}
                        />
                    </div>
                </div>
            }
        }
    };

    html! {
        <main class="hero">
            <div class="container">
                <div class="autopeer-page-header">
                    <h2 class="title title-flex">
                        <a href={(*autopeer_site_href).clone()} class="title-link">{i18n.t("app.title")}</a>
                        <span class="title-footnote">
                            {i18n.t("app.title.footnote")}
                            if !looking_glass_site_href.is_empty() {
                                {" / "}
                                <a href={(*looking_glass_site_href).clone()} class="autopeer-title-nav">{i18n.t("nav.looking_glass")}</a>
                            }
                        </span>
                    </h2>
                    <div class="autopeer-language-control">
                        <span class="autopeer-language-label">{i18n.t("nav.language")}</span>
                        <ShellSelect
                            value={locale_value}
                            on_change={on_locale_change}
                            aria_label={i18n.t("nav.language")}
                        >
                            {for Locale::ALL.iter().copied().map(|locale| html! {
                                <option value={locale.code()}>{locale.label()}</option>
                            })}
                        </ShellSelect>
                    </div>
                </div>
                <section class="autopeer">
                    <div class="autopeer-container">
                        {content}
                    </div>
                </section>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::autopeer_node_endpoint_port;
    use crate::{
        models::{MpBgpTransport, PeeringStrategy},
        store::{SessionDraft, SessionDraftField},
        update_form::{
            Peer6AddressKind, SessionDraftLiveValidation, SessionDraftToggleGroup,
            detect_peer6_address_kind, displayed_node_ipv4_visibility,
            displayed_peer6_address_kind, session_details_live_validation,
            session_details_submission_error, should_display_node_ipv4, should_mark_field_invalid,
        },
    };

    #[test]
    fn detects_link_local_peer6_addresses() {
        assert_eq!(
            detect_peer6_address_kind(" fe80::1234 "),
            Some(Peer6AddressKind::LinkLocal)
        );
    }

    #[test]
    fn detects_ula_peer6_addresses() {
        assert_eq!(
            detect_peer6_address_kind("fd42:4242:1023:68::1"),
            Some(Peer6AddressKind::Ula)
        );
    }

    #[test]
    fn rejects_invalid_peer6_prefix_matches() {
        assert_eq!(detect_peer6_address_kind("fe80::45455"), None);
    }

    #[test]
    fn keeps_last_committed_peer6_kind_while_peer6_is_focused() {
        assert_eq!(
            displayed_peer6_address_kind(
                "fd42:4242:1023:68::1",
                Some(SessionDraftField::Peer6),
                Some(Peer6AddressKind::LinkLocal),
            ),
            Some(Peer6AddressKind::LinkLocal)
        );
        assert_eq!(
            displayed_peer6_address_kind(
                "fe80::45455",
                Some(SessionDraftField::Peer6),
                Some(Peer6AddressKind::LinkLocal),
            ),
            Some(Peer6AddressKind::LinkLocal)
        );
    }

    #[test]
    fn derives_node_endpoint_port_from_peer_asn() {
        assert_eq!(autopeer_node_endpoint_port("4242421023"), "21023");
    }

    #[test]
    fn live_validation_flags_ipv4_only_without_peer4_after_toggle_interaction() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };

        let only_toggle = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);
        let validation = session_details_live_validation(&draft, &only_toggle, None, None);

        assert_eq!(validation.peer4_message, None);
        assert!(validation.highlight_ipv4);

        let with_peer4 = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftField::Peer4.into(),
        ]);
        let validation = session_details_live_validation(&draft, &with_peer4, None, None);

        assert_eq!(
            validation.peer4_message,
            Some("validation.peer4.required_ipv4".into())
        );
        assert!(!validation.highlight_peer4);
        assert!(validation.highlight_ipv4);
        assert!(!validation.highlight_mp_bgp);
    }

    #[test]
    fn live_validation_does_not_require_peer6_for_mp_bgp_without_ipv6_or_enh() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            extended_next_hop: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftToggleGroup::Bgp.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_peer6);
        assert!(!validation.highlight_mp_bgp);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_highlights_enh_when_ipv6_transport_is_required() {
        let touched = BTreeSet::from([SessionDraftToggleGroup::Bgp.into()]);
        let draft = SessionDraft {
            peer4: "172.20.193.67".into(),
            ipv6: false,
            ..SessionDraft::default()
        };

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert!(validation.peer6_messages.is_empty());
        assert_eq!(
            validation.bgp_message,
            Some("validation.extended_next_hop.requires_ipv6_transport".to_string())
        );
        assert!(!validation.highlight_ipv6);
        assert!(!validation.highlight_mp_bgp);
        assert!(validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_shows_enh_error_after_peer4_is_filled() {
        let touched = BTreeSet::from([SessionDraftField::Peer4.into()]);
        let draft = SessionDraft {
            peer4: "172.21.11.11".into(),
            peer6: String::new(),
            ..SessionDraft::default()
        };

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.bgp_message,
            Some("validation.extended_next_hop.requires_ipv6_transport".to_string())
        );
        assert!(validation.peer6_messages.is_empty());
        assert!(validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_does_not_project_ipv6_peer6_error_onto_mp_bgp_toggle() {
        let touched = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);
        let draft = SessionDraft {
            peer4: "172.20.193.67".into(),
            mp_bgp: false,
            extended_next_hop: false,
            ..SessionDraft::default()
        };

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert!(validation.peer6_messages.is_empty());
        assert!(validation.highlight_ipv6);
        assert!(!validation.highlight_mp_bgp);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_shows_invalid_peer4_message_after_blur() {
        let draft = SessionDraft {
            peer4: "1.1.1.1".into(),
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer4.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.peer4_message,
            Some("validation.peer4.range".into())
        );
        assert!(validation.highlight_peer4);
    }

    #[test]
    fn shows_node_ipv4_hint_only_for_valid_peer4() {
        assert!(should_display_node_ipv4(&SessionDraft {
            peer4: "172.20.193.67".into(),
            ..SessionDraft::default()
        }));
        assert!(!should_display_node_ipv4(&SessionDraft {
            peer4: "1.1.1.1".into(),
            ..SessionDraft::default()
        }));
        assert!(!should_display_node_ipv4(&SessionDraft::default()));
    }

    #[test]
    fn keeps_last_committed_node_ipv4_visibility_while_peer4_is_focused() {
        assert!(displayed_node_ipv4_visibility(
            &SessionDraft {
                peer4: "1.1.1.1".into(),
                ..SessionDraft::default()
            },
            Some(SessionDraftField::Peer4),
            true,
        ));
        assert!(!displayed_node_ipv4_visibility(
            &SessionDraft {
                peer4: "172.20.193.67".into(),
                ..SessionDraft::default()
            },
            Some(SessionDraftField::Peer4),
            false,
        ));
    }

    #[test]
    fn invalid_field_boxes_ignore_missing_optional_values() {
        assert!(!should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::Peer4,
        ));
        assert!(!should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::Peer6,
        ));
        assert!(!should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::Keepalive,
        ));
        assert!(!should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::Mtu,
        ));
        assert!(!should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::Psk,
        ));
    }

    #[test]
    fn invalid_field_boxes_flag_empty_required_fields() {
        assert!(!should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::Endpoint,
        ));
        assert!(should_mark_field_invalid(
            &SessionDraft::default(),
            SessionDraftField::WgPublicKey,
        ));
    }

    #[test]
    fn invalid_field_boxes_still_flag_non_empty_invalid_values() {
        assert!(should_mark_field_invalid(
            &SessionDraft {
                peer4: "1.1.1.1".into(),
                ..SessionDraft::default()
            },
            SessionDraftField::Peer4,
        ));
        assert!(should_mark_field_invalid(
            &SessionDraft {
                peer6: "x.x.x.x".into(),
                ..SessionDraft::default()
            },
            SessionDraftField::Peer6,
        ));
    }

    #[test]
    fn live_validation_clears_optional_peer4_highlight_when_peer6_is_valid() {
        let draft = SessionDraft {
            peer6: "fe80::1023".into(),
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer4.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(validation.peer4_message, None);
        assert!(!validation.highlight_peer4);
    }

    #[test]
    fn live_validation_shows_invalid_peer6_message_after_blur() {
        let draft = SessionDraft {
            peer6: "x.x.x.x".into(),
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer6.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.invalid".to_string()]
        );
        assert!(validation.highlight_peer6);
    }

    #[test]
    fn live_validation_shows_generic_tunnel_requirement_when_both_addresses_blank() {
        let touched = BTreeSet::from([SessionDraftField::Peer4.into()]);

        let validation =
            session_details_live_validation(&SessionDraft::default(), &touched, None, None);

        assert_eq!(
            validation.tunnel_message,
            Some("validation.tunnel.required".into())
        );
        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_peer4);
        assert!(!validation.highlight_ipv6);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_uses_tunnel_section_error_when_no_tunnel_address_is_present() {
        let touched = BTreeSet::from([
            SessionDraftField::Peer6.into(),
            SessionDraftToggleGroup::Families.into(),
            SessionDraftToggleGroup::Bgp.into(),
        ]);

        let validation =
            session_details_live_validation(&SessionDraft::default(), &touched, None, None);

        assert_eq!(
            validation.tunnel_message,
            Some("validation.tunnel.required".into())
        );
        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_peer6);
        assert!(!validation.highlight_ipv6);
        assert!(!validation.highlight_mp_bgp);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_defers_peer6_message_until_peer6_is_touched() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            mp_bgp: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let only_families = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);

        let validation = session_details_live_validation(&draft, &only_families, None, None);

        assert!(validation.peer6_messages.is_empty());
        assert!(validation.highlight_ipv6);

        let with_peer6 = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftField::Peer6.into(),
        ]);

        let validation = session_details_live_validation(&draft, &with_peer6, None, None);

        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.required_ipv6".to_string()]
        );
        assert!(!validation.highlight_peer6);
        assert!(validation.highlight_ipv6);
    }

    #[test]
    fn live_validation_hides_ipv6_route_requirement_when_mp_bgp_is_on() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            extended_next_hop: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_peer6);
        assert!(!validation.highlight_ipv6);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_shows_bgp_message_for_ipv4_over_ipv6_transport_without_peer4_or_enh() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftToggleGroup::Bgp.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.bgp_message,
            Some("validation.ipv4_over_ipv6_transport.requires_peer4_or_enh".to_string())
        );
        assert!(validation.peer4_message.is_none());
        assert!(validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_shows_bgp_message_when_ipv4_toggled_with_ipv6_transport_and_no_enh() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftToggleGroup::Bgp.into(),
        ]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.bgp_message,
            Some("validation.ipv4_over_ipv6_transport.requires_peer4_or_enh".to_string())
        );
        assert!(validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_hides_link_local_error_while_own6_is_focused() {
        let draft = SessionDraft {
            peer6: "fe80::454:2".into(),
            own6: "11".into(),
            ..SessionDraft::default()
        };

        let validation = session_details_live_validation(
            &draft,
            &BTreeSet::new(),
            Some(SessionDraftField::Own6),
            None,
        );

        assert_eq!(validation.tunnel_message, None);
        assert!(!validation.highlight_own6);
    }

    #[test]
    fn live_validation_no_peer6_message_while_focused_if_not_yet_touched() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftToggleGroup::Bgp.into(),
        ]);

        let validation =
            session_details_live_validation(&draft, &touched, Some(SessionDraftField::Peer6), None);

        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_peer6);
        assert!(!validation.highlight_ipv6);
        assert!(!validation.highlight_mp_bgp);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_requires_peer6_for_explicit_ipv6_transport_after_blur() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer6.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.required_mp_bgp".to_string()]
        );
        assert_eq!(validation.tunnel_message, None);
        assert!(!validation.highlight_peer6);
        assert!(!validation.highlight_ipv6);
        assert!(!validation.highlight_mp_bgp);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_requires_peer4_for_explicit_ipv4_transport_after_blur() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv4: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv4),
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer4.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.peer4_message,
            Some("validation.peer4.required_mp_bgp".into())
        );
        assert_eq!(validation.tunnel_message, None);
        assert!(!validation.highlight_peer4);
        assert!(!validation.highlight_ipv4);
        assert!(!validation.highlight_mp_bgp);
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_keeps_peer6_message_while_focused() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer6.into()]);

        let validation =
            session_details_live_validation(&draft, &touched, Some(SessionDraftField::Peer6), None);

        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.required_mp_bgp".to_string()]
        );
        assert!(!validation.highlight_peer6);
    }

    #[test]
    fn live_validation_shows_enh_error_while_peer6_is_focused_after_clear() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            peer6: String::new(),
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftField::Peer6.into()]);

        let validation =
            session_details_live_validation(&draft, &touched, Some(SessionDraftField::Peer6), None);

        assert_eq!(
            validation.bgp_message,
            Some("validation.extended_next_hop.requires_ipv6_transport".to_string())
        );
        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_extended_next_hop);
    }

    #[test]
    fn live_validation_tunnel_message_requires_tunnel_field_touched() {
        let only_toggles = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftToggleGroup::Bgp.into(),
        ]);

        let validation =
            session_details_live_validation(&SessionDraft::default(), &only_toggles, None, None);

        assert_eq!(validation.tunnel_message, None);

        let with_peer4 = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftField::Peer4.into(),
        ]);

        let validation =
            session_details_live_validation(&SessionDraft::default(), &with_peer4, None, None);

        assert_eq!(
            validation.tunnel_message,
            Some("validation.tunnel.required".into())
        );
    }

    #[test]
    fn live_validation_field_errors_only_after_own_field_blur() {
        let draft = SessionDraft {
            peer4: "172.20.193.67".into(),
            mp_bgp: false,
            extended_next_hop: false,
            ..SessionDraft::default()
        };

        let only_toggles = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);
        let validation = session_details_live_validation(&draft, &only_toggles, None, None);
        assert!(validation.peer6_messages.is_empty());
        assert!(validation.highlight_ipv6);

        let with_peer6 = BTreeSet::from([
            SessionDraftToggleGroup::Families.into(),
            SessionDraftField::Peer6.into(),
        ]);
        let validation = session_details_live_validation(&draft, &with_peer6, None, None);
        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.required_ipv6".to_string()]
        );
    }

    #[test]
    fn live_validation_focus_does_not_change_other_fields() {
        let draft = SessionDraft::default();
        let touched = BTreeSet::from([
            SessionDraftField::Peer4.into(),
            SessionDraftField::Peer6.into(),
        ]);

        let unfocused = session_details_live_validation(&draft, &touched, None, None);

        let focused_peer4 =
            session_details_live_validation(&draft, &touched, Some(SessionDraftField::Peer4), None);

        assert_eq!(unfocused.tunnel_message, focused_peer4.tunnel_message);
        assert_eq!(unfocused.peer6_messages, focused_peer4.peer6_messages);
    }

    #[test]
    fn live_validation_stacks_section_and_field_errors() {
        let draft = SessionDraft {
            mp_bgp: false,
            extended_next_hop: false,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([
            SessionDraftField::Peer4.into(),
            SessionDraftField::Peer6.into(),
        ]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.tunnel_message,
            Some("validation.tunnel.required".into())
        );
        assert!(
            validation.peer4_message.is_some() || validation.peer6_messages.iter().any(|_| true),
            "at least one field-level error should coexist with the section error"
        );
    }

    #[test]
    fn live_validation_keeps_peer6_messages_out_of_tunnel_section_error() {
        let touched = BTreeSet::from([SessionDraftField::Peer6.into()]);

        let validation =
            session_details_live_validation(&SessionDraft::default(), &touched, None, None);

        assert_eq!(
            validation.tunnel_message,
            Some("validation.tunnel.required".into())
        );
        assert!(validation.peer6_messages.is_empty());
        assert!(!validation.highlight_peer6);
        assert!(!validation.highlight_peer4);
    }

    #[test]
    fn live_validation_marks_both_fields_for_placeholder_link_local_collision() {
        let draft = SessionDraft {
            peer6: "fe80::1023:2".into(),
            ..SessionDraft::default()
        };

        let validation =
            session_details_live_validation(&draft, &BTreeSet::new(), None, Some("fe80::1023:2"));

        assert!(validation.peer6_messages.is_empty());
        assert_eq!(
            validation.own6_message,
            Some("validation.own6.must_differ_from_peer6".into())
        );
        assert!(validation.highlight_peer6);
        assert!(validation.highlight_own6);
    }

    #[test]
    fn submission_error_rejects_placeholder_link_local_collision() {
        let draft = SessionDraft {
            peer6: "fe80::1023:2".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            session_details_submission_error(&draft, Some("fe80::1023:2")),
            Some("validation.own6.must_differ_from_peer6".into())
        );
    }

    #[test]
    fn live_validation_stays_quiet_for_untouched_default_draft() {
        let validation =
            session_details_live_validation(&SessionDraft::default(), &BTreeSet::new(), None, None);

        assert_eq!(validation, SessionDraftLiveValidation::default());
    }
}
