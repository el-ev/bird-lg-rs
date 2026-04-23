use common::models::PeeringInfo;
use ui_components::shell::{
    ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect, ShellToggle,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::{
    controller::{
        AutoPeerController, OngoingTask, default_pgp_key, selected_registry_email_target,
        sync_create_draft, use_autopeer_controller,
    },
    i18n::{I18n, Locale, use_i18n},
    models::{
        ALL_MP_BGP_TRANSPORTS, ALL_PEERING_STRATEGIES, AuthMethodKind, MpBgpTransport, NodeView,
        OperationFailureStage, OperationKind, OperationState, OperationStatus, PeeringStrategy,
        SessionState, UiMessage,
    },
    store::{AutoPeerStep, PeerConfigStage, SessionDraft, SessionDraftField},
    update_form::{
        Peer6AddressKind, SessionDraftToggleGroup, SessionDraftTouchedControls,
        detect_peer6_address_kind, displayed_node_ipv4_visibility, displayed_peer6_address_kind,
        field_is_touched, session_details_live_validation, session_details_submission_error,
        should_display_node_ipv4, should_mark_field_invalid, touch_field, touch_toggle_group,
    },
};

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

fn update_draft_state(
    draft: &UseStateHandle<SessionDraft>,
    update: impl FnOnce(&mut SessionDraft),
) {
    let mut next = (**draft).clone();
    update(&mut next);
    draft.set(next);
}

fn update_touched_controls(
    touched_fields: &UseStateHandle<SessionDraftTouchedControls>,
    update: impl FnOnce(&mut SessionDraftTouchedControls),
) {
    let mut next = (**touched_fields).clone();
    update(&mut next);
    touched_fields.set(next);
}

fn live_validation_message(i18n: &I18n, message: Option<&str>) -> Html {
    match message {
        Some(message) => html! {
            <p class="autopeer-live-validation" aria-live="polite">{i18n.translate_owned(message)}</p>
        },
        None => Html::default(),
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
    if let Some(error) = error {
        html! {
            <ShellLine>
                <span class="error-message">{i18n.translate_message(error)}</span>
            </ShellLine>
        }
    } else {
        Html::default()
    }
}

fn render_loading(i18n: &I18n, loading: bool, loading_message: Option<UiMessage>) -> Html {
    if !loading {
        return Html::default();
    }
    let message = match loading_message {
        Some(message) => i18n.translate_message(&message),
        None => i18n.t("status.working").to_string(),
    };
    html! {
        <div class="autopeer-ongoing-task">
            <ShellLine>
                <span class="text-secondary">{message}</span>
            </ShellLine>
        </div>
    }
}

fn render_ongoing_tasks(i18n: &I18n, tasks: &[OngoingTask]) -> Html {
    if tasks.is_empty() {
        return Html::default();
    }
    html! {
        <div class="autopeer-ongoing-tasks">
            { for tasks.iter().map(|task| {
                let message = i18n.translate_message(&task.message);
                html! {
                    <div class="autopeer-ongoing-task" key={task.id}>
                        <ShellLine>
                            <span class="text-secondary">{message}</span>
                        </ShellLine>
                    </div>
                }
            }) }
        </div>
    }
}

fn looking_glass_href_from_parts(protocol: &str, host: &str) -> String {
    if let Some(rest) = host.strip_prefix("autopeer.") {
        format!("{protocol}//network.{rest}/")
    } else {
        format!("{protocol}//{host}/")
    }
}

fn looking_glass_href() -> String {
    web_sys::window()
        .and_then(|window| {
            let location = window.location();
            let protocol = location.protocol().ok()?;
            let host = location.host().ok()?;
            Some(looking_glass_href_from_parts(&protocol, &host))
        })
        .unwrap_or_else(|| "/".to_string())
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
        _ => i18n.t("node.transport.dual_stack"),
    }
}

fn mp_bgp_transport_label(i18n: &I18n, transport: MpBgpTransport) -> &'static str {
    humanize_ip_support(i18n, transport.as_str())
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

fn node_review_line(i18n: &I18n, node: &NodeView) -> String {
    let context = node_context_line(i18n, node);
    if context.is_empty() {
        node.name.clone()
    } else {
        format!("{} ({context})", node.name)
    }
}

fn session_state_label(i18n: &I18n, state: &SessionState) -> &'static str {
    match state {
        SessionState::Managed => i18n.t("session_state.managed"),
        SessionState::Manual => i18n.t("session_state.manual"),
        SessionState::PendingPr => i18n.t("session_state.pending_pr"),
        SessionState::StalledPr => i18n.t("session_state.stalled_pr"),
        SessionState::Conflict => i18n.t("session_state.conflict"),
    }
}

fn operation_kind_label(i18n: &I18n, kind: &OperationKind) -> &'static str {
    match kind {
        OperationKind::Create => i18n.t("operation.kind.create"),
        OperationKind::Update => i18n.t("operation.kind.update"),
        OperationKind::Retire => i18n.t("operation.kind.retire"),
        OperationKind::Delete => i18n.t("operation.kind.delete"),
        OperationKind::Migrate => i18n.t("operation.kind.migrate"),
    }
}

fn operation_state_label(i18n: &I18n, state: &OperationState) -> &'static str {
    match state {
        OperationState::PendingPullRequest => i18n.t("operation.state.pending_pull_request"),
        OperationState::PendingChecks => i18n.t("operation.state.pending_checks"),
        OperationState::Applying => i18n.t("operation.state.applying"),
        OperationState::PendingMerge => i18n.t("operation.state.pending_merge"),
        OperationState::Completed => i18n.t("operation.state.completed"),
        OperationState::Failed => i18n.t("operation.state.failed"),
        OperationState::Conflict => i18n.t("operation.state.conflict"),
    }
}

fn operation_failure_stage_label(i18n: &I18n, stage: &OperationFailureStage) -> &'static str {
    match stage {
        OperationFailureStage::Checks => i18n.t("operation.failure_stage.checks"),
        OperationFailureStage::Preflight => i18n.t("operation.failure_stage.preflight"),
        OperationFailureStage::Apply => i18n.t("operation.failure_stage.apply"),
        OperationFailureStage::Merge => i18n.t("operation.failure_stage.merge"),
    }
}

fn peering_strategy_label(i18n: &I18n, strategy: PeeringStrategy) -> &'static str {
    match strategy {
        PeeringStrategy::FullTable => i18n.t("peering_strategy.full_table.label"),
        PeeringStrategy::Transit => i18n.t("peering_strategy.transit.label"),
        PeeringStrategy::Peer => i18n.t("peering_strategy.peer.label"),
        PeeringStrategy::Downstream => i18n.t("peering_strategy.downstream.label"),
    }
}

fn peering_strategy_description(i18n: &I18n, strategy: PeeringStrategy) -> &'static str {
    match strategy {
        PeeringStrategy::FullTable => i18n.t("peering_strategy.full_table.description"),
        PeeringStrategy::Transit => i18n.t("peering_strategy.transit.description"),
        PeeringStrategy::Peer => i18n.t("peering_strategy.peer.description"),
        PeeringStrategy::Downstream => i18n.t("peering_strategy.downstream.description"),
    }
}

fn has_peering_info(peering: &PeeringInfo) -> bool {
    peering.ipv4.is_some()
        || peering.ipv6.is_some()
        || peering.link_local_ipv6.is_some()
        || peering.wg_pubkey.is_some()
        || peering.endpoint.is_some()
        || peering.comment.is_some()
}

fn review_item(label: &'static str, value: String) -> Html {
    html! {
        <div class="autopeer-review-item">
            <span class="autopeer-review-label">{label}</span>
            <strong class="autopeer-review-value">{value}</strong>
        </div>
    }
}

fn optional_review_item(label: &'static str, value: &str) -> Html {
    match value.trim() {
        "" => Html::default(),
        value => review_item(label, value.to_string()),
    }
}

fn render_peering_field(label: &'static str, value: Option<&str>) -> Html {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => html! {
            <>
                <dt class="peering-label">{label}</dt>
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

fn render_inventory_peering_review(i18n: &I18n, node: Option<&NodeView>, active_asn: &str) -> Html {
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
    if !has_peering_info(peering) && node_endpoint.is_none() {
        return Html::default();
    }

    html! {
        <div class="autopeer-review-section">
            <p class="autopeer-review-section-title">{i18n.t("stage3.review.our_node_details")}</p>
            <dl class="peering-grid autopeer-review-peering-grid">
                {render_peering_field(i18n.t("stage3.review.our_endpoint"), node_endpoint.as_deref())}
                {render_peering_field(i18n.t("stage3.review.our_ipv4"), peering.ipv4.as_deref())}
                {render_peering_field(i18n.t("stage3.review.our_ipv6"), peering.ipv6.as_deref())}
                {render_peering_field(i18n.t("stage3.review.our_link_local_ipv6"), peering.link_local_ipv6.as_deref())}
                {render_peering_field(i18n.t("stage3.review.our_wg_pubkey"), peering.wg_pubkey.as_deref())}
                {render_peering_field(i18n.t("stage3.review.our_node_note"), peering.comment.as_deref())}
            </dl>
        </div>
    }
}

fn operation_stage_index(operation: &OperationStatus) -> usize {
    match operation.state {
        OperationState::PendingPullRequest => 0,
        OperationState::PendingChecks => 1,
        OperationState::Applying => 2,
        OperationState::PendingMerge => 3,
        OperationState::Completed => 4,
        OperationState::Failed | OperationState::Conflict => operation
            .failure_details
            .as_ref()
            .map(|d| match d.stage {
                OperationFailureStage::Checks => 1,
                OperationFailureStage::Preflight | OperationFailureStage::Apply => 2,
                OperationFailureStage::Merge => 3,
            })
            .unwrap_or(2),
    }
}

fn displayed_peer_config_stage(
    editing_node: Option<&str>,
    config_stage: PeerConfigStage,
) -> PeerConfigStage {
    if editing_node.is_some() && config_stage == PeerConfigStage::SelectNode {
        PeerConfigStage::SessionDetails
    } else {
        config_stage
    }
}

fn retire_button_text(i18n: &I18n, retire_confirmation: bool) -> &'static str {
    if retire_confirmation {
        i18n.t("action.confirm_retirement")
    } else {
        i18n.t("action.retire_session")
    }
}

fn delete_button_text(i18n: &I18n, delete_confirmation: bool) -> &'static str {
    if delete_confirmation {
        i18n.t("action.confirm_deletion")
    } else {
        i18n.t("action.delete_session")
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
    let active_index = operation_stage_index(operation);
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

#[function_component(AutoPeerPage)]
pub fn auto_peer_page() -> Html {
    let i18n = use_i18n();
    let default_autopeer_home_href = String::from("/");
    let default_looking_glass_href = looking_glass_href();
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
            let target: HtmlSelectElement = event.target_unchecked_into();
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
                {render_loading(&i18n, true, Some(UiMessage::key("step.loading_config.message")))}
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
            let selected_method_value = (*selected_method).clone();
            if let Some(method) = selected_method_value {
                let method_label = i18n.translate_message(&method.label);
                let verification_fields = match method.kind {
                    AuthMethodKind::RegistrySsh => {
                        let on_change = {
                            let ssh_signature = ssh_signature.clone();
                            Callback::from(move |value: String| ssh_signature.set(value))
                        };
                        html! {
                            <>
                                if method.ssh_fingerprints.is_empty() {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {i18n.t("verify.ssh.no_fingerprints")}
                                        </span>
                                    </ShellLine>
                                } else if method.ssh_fingerprints.len() == 1 {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.key")}</ShellPrompt>
                                        {" "}{i18n.translate_params(
                                            "verify.ssh.match_one",
                                            &[("fingerprint", method.ssh_fingerprints[0].as_str())],
                                        )}
                                    </ShellLine>
                                } else {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.keys")}</ShellPrompt>
                                        {" "} {{
                                            let fingerprints = method.ssh_fingerprints.join(", ");
                                            i18n.translate_params(
                                                "verify.ssh.match_many",
                                                &[("fingerprints", fingerprints.as_str())],
                                            )
                                        }}
                                    </ShellLine>
                                }
                                if let Some(challenge) = &*challenge_text {
                                    {render_readonly_block(
                                        i18n.t("verify.ssh.create_signature"),
                                        ssh_sign_command(challenge),
                                    )}
                                }
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("prompt.signature")}</ShellPrompt>
                                    {" "}{i18n.t("verify.ssh.paste_prompt")}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*ssh_signature).clone()}
                                        on_change={on_change}
                                        placeholder={i18n.t("verify.ssh.placeholder")}
                                        disabled={loading}
                                        multiline=true
                                        rows={10}
                                    />
                                </ShellLine>
                            </>
                        }
                    }
                    AuthMethodKind::RegistryPgp => {
                        let on_pubkey_change = {
                            let pgp_public_key = pgp_public_key.clone();
                            Callback::from(move |value: String| pgp_public_key.set(value))
                        };
                        let on_signed_change = {
                            let pgp_signed_message = pgp_signed_message.clone();
                            Callback::from(move |value: String| pgp_signed_message.set(value))
                        };
                        let selected_key_value = if (*selected_pgp_key).is_empty() {
                            default_pgp_key(&method)
                        } else {
                            (*selected_pgp_key).clone()
                        };
                        let on_key_change = {
                            let selected_pgp_key = selected_pgp_key.clone();
                            Callback::from(move |event: Event| {
                                let select: HtmlSelectElement = event.target_unchecked_into();
                                selected_pgp_key.set(select.value());
                            })
                        };

                        html! {
                            <>
                                if method.pgp_fingerprints.is_empty() {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {i18n.t("verify.pgp.no_fingerprints")}
                                        </span>
                                    </ShellLine>
                                } else if method.pgp_fingerprints.len() == 1 {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.key")}</ShellPrompt>
                                        {" "}{i18n.translate_params(
                                            "verify.pgp.use_key",
                                            &[("fingerprint", method.pgp_fingerprints[0].as_str())],
                                        )}
                                    </ShellLine>
                                } else {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.key")}</ShellPrompt>
                                        {" "}
                                        <ShellSelect value={selected_key_value.clone()} on_change={on_key_change}>
                                            {for method.pgp_fingerprints.iter().map(|fingerprint| html! {
                                                <option value={fingerprint.clone()}>{fingerprint.clone()}</option>
                                            })}
                                        </ShellSelect>
                                    </ShellLine>
                                }
                                if let Some(challenge) = &*challenge_text {
                                    <>
                                        <ShellLine>
                                            <span class="text-secondary">
                                                {i18n.t("verify.pgp.clearsign_intro")}
                                            </span>
                                        </ShellLine>
                                        {render_readonly_block(
                                            i18n.t("verify.pgp.exact_challenge"),
                                            challenge.clone(),
                                        )}
                                        {render_readonly_block(
                                            i18n.t("verify.pgp.clearsign_label"),
                                            pgp_sign_command(challenge, &selected_key_value),
                                        )}
                                    </>
                                } else {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {i18n.t("verify.pgp.clearsign_intro")}
                                        </span>
                                    </ShellLine>
                                }
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("prompt.signed")}</ShellPrompt>
                                    {" "}{i18n.t("verify.pgp.signed_paste_prompt")}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*pgp_signed_message).clone()}
                                        on_change={on_signed_change}
                                        placeholder={i18n.t("verify.pgp.signed_placeholder")}
                                        disabled={loading}
                                        multiline=true
                                        rows={12}
                                    />
                                </ShellLine>
                                {render_readonly_block(
                                    i18n.t("verify.pgp.export_label"),
                                    pgp_export_command(&selected_key_value),
                                )}
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("prompt.pubkey")}</ShellPrompt>
                                    {" "}{i18n.t("verify.pgp.pubkey_paste_prompt")}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*pgp_public_key).clone()}
                                        on_change={on_pubkey_change}
                                        placeholder={i18n.t("verify.pgp.pubkey_placeholder")}
                                        disabled={loading}
                                        multiline=true
                                        rows={8}
                                    />
                                </ShellLine>
                            </>
                        }
                    }
                    AuthMethodKind::RegistryEmail => {
                        let selected_target = selected_registry_email_target(
                            &method,
                            selected_email_maintainer.as_str(),
                        );
                        let selected_target_value = selected_target
                            .map(|target| target.maintainer.clone())
                            .unwrap_or_else(|| (*selected_email_maintainer).clone());
                        let on_target_change = {
                            let on_selected_email_maintainer_change =
                                on_selected_email_maintainer_change.clone();
                            Callback::from(move |event: Event| {
                                let select: HtmlSelectElement = event.target_unchecked_into();
                                on_selected_email_maintainer_change.emit(select.value());
                            })
                        };
                        let on_code_change = on_registry_email_code_change.clone();
                        let send_button_text = if registry_email_sent_to.is_empty() {
                            i18n.t("action.send_signin_link")
                        } else {
                            i18n.t("action.resend_signin_link")
                        };
                        let selected_target_emails =
                            selected_target.map(|target| target.emails.join(", "));

                        html! {
                            <>
                                <ShellLine>
                                    <span class="text-secondary">
                                        {i18n.t("verify.email.intro")}
                                    </span>
                                </ShellLine>
                                if method.email_targets.is_empty() {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {i18n.t("verify.email.no_contacts")}
                                        </span>
                                    </ShellLine>
                                } else if method.email_targets.len() == 1 {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.mntner")}</ShellPrompt>
                                        {" "}{i18n.translate_params(
                                            "verify.email.auth_as",
                                            &[("mnt", method.email_targets[0].maintainer.as_str())],
                                        )}
                                    </ShellLine>
                                } else {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.mntner")}</ShellPrompt>
                                        {" "}
                                        <ShellSelect
                                            value={selected_target_value.clone()}
                                            on_change={on_target_change}
                                        >
                                            {for method.email_targets.iter().map(|target| html! {
                                                <option value={target.maintainer.clone()}>{target.maintainer.clone()}</option>
                                            })}
                                        </ShellSelect>
                                    </ShellLine>
                                }
                                if let Some(emails) = &selected_target_emails {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.emails")}</ShellPrompt>
                                        {" "}{i18n.translate_params(
                                            "verify.email.send_to",
                                            &[("emails", emails.as_str())],
                                        )}
                                    </ShellLine>
                                }
                                if !registry_email_sent_to.is_empty() {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {{
                                                let emails = registry_email_sent_to.join(", ");
                                                i18n.translate_params(
                                                    "verify.email.sent_to_prefix",
                                                    &[("emails", emails.as_str())],
                                                )
                                            }}
                                        </span>
                                    </ShellLine>
                                }
                                <ShellLine>
                                    <ShellButton
                                        text={send_button_text}
                                        onclick={on_send_registry_email.clone()}
                                        disabled={loading || selected_target.is_none()}
                                    />
                                </ShellLine>
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("prompt.code")}</ShellPrompt>
                                    {" "}{i18n.t("verify.email.code_prompt")}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*registry_email_code).clone()}
                                        on_change={on_code_change}
                                        placeholder={i18n.t("verify.email.code_placeholder")}
                                        disabled={loading}
                                    />
                                </ShellLine>
                            </>
                        }
                    }
                    AuthMethodKind::Oidc => {
                        html! {
                            <>
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("prompt.login")}</ShellPrompt>
                                    {" "}{i18n.translate_params(
                                        "verify.oidc.in_browser",
                                        &[("provider", method_label.as_str())],
                                    )}
                                </ShellLine>
                                <ShellLine>
                                    <span class="text-secondary">
                                        {i18n.t("verify.oidc.redirect_note")}
                                    </span>
                                </ShellLine>
                            </>
                        }
                    }
                    AuthMethodKind::HostImpersonation => html! {
                        <ShellLine>
                            <span class="text-secondary">
                                {i18n.t("verify.host.note")}
                            </span>
                        </ShellLine>
                    },
                };
                let verify_button_text = if method.kind == AuthMethodKind::Oidc {
                    i18n.translate_params(
                        "verify.oidc.continue_to",
                        &[("provider", method_label.as_str())],
                    )
                } else if method.kind == AuthMethodKind::RegistryEmail {
                    i18n.t("action.verify_code").to_string()
                } else {
                    i18n.t("action.verify").to_string()
                };

                html! {
                    <div class="autopeer-step">
                        <ShellLine>
                            <ShellPrompt>{i18n.t("prompt.auth")}</ShellPrompt>
                            {" "}{i18n.translate_params(
                                "verify.auth_for_as",
                                &[
                                    ("label", method_label.as_str()),
                                    ("asn", asn.as_str()),
                                ],
                            )}
                        </ShellLine>
                        {verification_fields}
                        {render_ongoing_tasks(&i18n, ongoing_tasks.tasks())}
                        {render_error(&i18n, &error)}
                        <ShellLine>
                            <ShellButton
                                text={i18n.t("action.back")}
                                onclick={on_verify_back.clone()}
                                disabled={loading}
                            />
                            {" "}
                            <ShellButton text={verify_button_text} onclick={on_verify} disabled={loading} />
                        </ShellLine>
                    </div>
                }
            } else {
                html! {
                    <div class="autopeer-step">
                        <ShellLine>
                            <span class="error-message">{i18n.t("verify.choose_first")}</span>
                        </ShellLine>
                    </div>
                }
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
                displayed_peer_config_stage(editing_node_value.as_deref(), *config_stage);
            let selected_node_name = editing_node_value.as_deref().or_else(|| {
                let selected = draft.node.trim();
                (!selected.is_empty()).then_some(selected)
            });
            let selected_node = selected_node_name
                .and_then(|name| nodes.iter().find(|node| node.name == name).cloned());
            let selected_session = selected_node_name
                .and_then(|name| sessions.iter().find(|s| s.node == name));
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
            let live_validation = session_details_live_validation(
                &draft,
                &touched_fields,
                *focused_field,
                node_inventory_link_local_ipv6.as_deref(),
            );
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

            let on_cancel_edit = {
                let editing_node = editing_node.clone();
                let draft = draft.clone();
                let sessions = sessions.clone();
                let nodes = nodes.clone();
                let config_stage = config_stage.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    editing_node.set(None);
                    config_stage.set(PeerConfigStage::SelectNode);
                    touched_fields.set(SessionDraftTouchedControls::new());
                    draft.set(sync_create_draft(&nodes, &sessions, &draft));
                })
            };

            let update_text_field = |setter: fn(&mut SessionDraft) -> &mut String| {
                let draft = draft.clone();
                Callback::from(move |value: String| {
                    update_draft_state(&draft, |next| *setter(next) = value)
                })
            };

            let on_field_blur = |field: SessionDraftField| {
                let touched_fields = touched_fields.clone();
                let focused_field = focused_field.clone();
                let target_field = field;
                Callback::from(move |_: FocusEvent| {
                    if *focused_field == Some(target_field) {
                        focused_field.set(None);
                    }
                    update_touched_controls(&touched_fields, |next| {
                        touch_field(next, target_field)
                    });
                })
            };

            let on_field_focus = |field: SessionDraftField| {
                let focused_field = focused_field.clone();
                Callback::from(move |_: FocusEvent| focused_field.set(Some(field)))
            };

            let on_peer6_blur = {
                let touched_fields = touched_fields.clone();
                let focused_field = focused_field.clone();
                let committed_peer6_kind = committed_peer6_kind.clone();
                let draft = draft.clone();
                Callback::from(move |_: FocusEvent| {
                    if *focused_field == Some(SessionDraftField::Peer6) {
                        focused_field.set(None);
                    }
                    update_touched_controls(&touched_fields, |next| {
                        touch_field(next, SessionDraftField::Peer6);
                    });
                    let next_kind = detect_peer6_address_kind(&draft.peer6);
                    committed_peer6_kind.set(next_kind);
                    if next_kind != Some(Peer6AddressKind::LinkLocal) && !draft.own6.is_empty() {
                        update_draft_state(&draft, |next| next.own6.clear());
                    }
                })
            };

            let field_is_invalid = |field: SessionDraftField| {
                field_is_touched(&touched_fields, field)
                    && *focused_field != Some(field)
                    && should_mark_field_invalid(&draft, field)
            };

            let input_class = |field: SessionDraftField| {
                if field_is_invalid(field) || live_validation.highlights_field(field) {
                    classes!("shell-input--invalid")
                } else {
                    Classes::new()
                }
            };

            let input_frame_class = |field: SessionDraftField| {
                if field_is_invalid(field) || live_validation.highlights_field(field) {
                    classes!("shell-input-frame--invalid")
                } else {
                    Classes::new()
                }
            };

            let field_error_message = |field: SessionDraftField| {
                if field_is_invalid(field) {
                    live_validation_message(&i18n, draft.field_error(field).as_deref())
                } else {
                    Html::default()
                }
            };

            let toggle_item_class =
                |invalid: bool| classes!("autopeer-toggle-item", invalid.then_some("is-invalid"));

            let on_peer6_change = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |value: String| {
                    if value.trim().is_empty() {
                        update_touched_controls(&touched_fields, |next| {
                            touch_field(next, SessionDraftField::Peer6);
                        });
                    }
                    update_draft_state(&draft, |next| next.peer6 = value)
                })
            };

            let on_toggle_ipv4 = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    update_touched_controls(&touched_fields, |next| {
                        touch_toggle_group(next, SessionDraftToggleGroup::Families);
                    });
                    update_draft_state(&draft, |next| next.ipv4 = !next.ipv4);
                })
            };

            let on_toggle_ipv6 = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    update_touched_controls(&touched_fields, |next| {
                        touch_toggle_group(next, SessionDraftToggleGroup::Families);
                    });
                    update_draft_state(&draft, |next| next.ipv6 = !next.ipv6);
                })
            };

            let on_toggle_mp_bgp = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_: ()| {
                    update_touched_controls(&touched_fields, |next| {
                        touch_toggle_group(next, SessionDraftToggleGroup::Bgp);
                    });
                    update_draft_state(&draft, |next| {
                        next.mp_bgp = !next.mp_bgp;
                        if !next.mp_bgp {
                            next.extended_next_hop = false;
                        }
                    });
                })
            };

            let on_toggle_extended_next_hop = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    update_touched_controls(&touched_fields, |next| {
                        touch_toggle_group(next, SessionDraftToggleGroup::Bgp);
                    });
                    update_draft_state(&draft, |next| {
                        next.extended_next_hop = !next.extended_next_hop;
                        if next.extended_next_hop {
                            next.mp_bgp = true;
                            next.ipv4 = true;
                            next.mp_bgp_transport = Some(MpBgpTransport::Ipv6);
                        }
                    });
                })
            };

            let on_change_mp_bgp_transport = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |event: Event| {
                    let select: HtmlSelectElement = event.target_unchecked_into();
                    let value = select.value();
                    update_touched_controls(&touched_fields, |next| {
                        touch_toggle_group(next, SessionDraftToggleGroup::Bgp);
                    });
                    update_draft_state(&draft, |next| {
                        next.mp_bgp_transport = MpBgpTransport::from_value(&value);
                        if next.mp_bgp_transport == Some(MpBgpTransport::Ipv4) {
                            next.extended_next_hop = false;
                        }
                    });
                })
            };

            let on_change_peering_strategy = {
                let draft = draft.clone();
                Callback::from(move |event: Event| {
                    let select: HtmlSelectElement = event.target_unchecked_into();
                    let value = select.value();
                    update_draft_state(&draft, |next| {
                        next.peering_strategy = PeeringStrategy::from_value(&value)
                            .unwrap_or(PeeringStrategy::FullTable);
                    });
                })
            };

            let on_toggle_encrypt_endpoint = {
                let draft = draft.clone();
                Callback::from(move |_| {
                    update_draft_state(&draft, |next| next.encrypt_endpoint = !next.encrypt_endpoint);
                })
            };

            let on_psk_action = {
                let draft = draft.clone();
                let psk_copied = psk_copied.clone();
                Callback::from(move |_: MouseEvent| {
                    if draft.has_psk {
                        update_draft_state(&draft, |next| {
                            next.clear_psk = true;
                            next.has_psk = false;
                            next.psk.clear();
                        });
                    } else if !draft.psk.is_empty() {
                        update_draft_state(&draft, |next| {
                            next.psk.clear();
                        });
                    } else if let Some(key) = generate_wg_psk() {
                        let draft = draft.clone();
                        let psk_copied = psk_copied.clone();
                        update_draft_state(&draft, |next| {
                            next.psk = key.clone();
                        });
                        if let Some(window) = web_sys::window() {
                            let clipboard = window.navigator().clipboard();
                            let psk_copied_inner = psk_copied.clone();
                            spawn_local(async move {
                                let _ = wasm_bindgen_futures::JsFuture::from(
                                    clipboard.write_text(&key),
                                )
                                .await;
                                psk_copied_inner.set(true);
                                gloo_timers::future::TimeoutFuture::new(2_000).await;
                                psk_copied_inner.set(false);
                            });
                        }
                    }
                })
            };

            let on_back_to_details = {
                let config_stage = config_stage.clone();
                Callback::from(move |_| config_stage.set(PeerConfigStage::SessionDetails))
            };

            let on_change_node = {
                let config_stage = config_stage.clone();
                Callback::from(move |_| config_stage.set(PeerConfigStage::SelectNode))
            };

            let on_step_click = {
                let config_stage = config_stage.clone();
                Callback::from(move |stage: PeerConfigStage| config_stage.set(stage))
            };

            let on_continue_to_review = {
                let draft = draft.clone();
                let editing_node = editing_node.clone();
                let config_stage = config_stage.clone();
                let error = error.clone();
                Callback::from(move |_| {
                    if editing_node.is_none() && draft.node.trim().is_empty() {
                        error.set(Some(UiMessage::key("error.ui.node.choose")));
                        return;
                    }

                    match session_details_submission_error(
                        &draft,
                        node_inventory_link_local_ipv6.as_deref(),
                    ) {
                        None => {
                            error.set(None);
                            config_stage.set(PeerConfigStage::Review);
                        }
                        Some(message) => error.set(Some(UiMessage::key(message))),
                    }
                })
            };

            let main_panel = match active_stage {
                PeerConfigStage::SelectNode => html! {
                    <article class="peering-card autopeer-panel">
                        <div class="autopeer-panel-header">
                            <p class="autopeer-panel-kicker">{i18n.t("stage1.kicker")}</p>
                            <h3 class="autopeer-panel-title">{i18n.t("stage1.title")}</h3>
                            <p class="text-secondary">
                                {i18n.t("stage1.description")}
                            </p>
                        </div>
                        if nodes.is_empty() {
                            <div class="autopeer-empty-state">
                                <p>{i18n.t("stage1.empty_title")}</p>
                                <p class="text-secondary">
                                    {i18n.t("stage1.empty_body")}
                                </p>
                            </div>
                        } else {
                            <div class="autopeer-node-grid">
                                {for nodes.iter().map(|node| {
                                    let node_session = sessions.iter().find(|session| session.node == node.name).cloned();
                                    let node_session_for_click = node_session.clone();
                                    let draft = draft.clone();
                                    let editing_node = editing_node.clone();
                                    let config_stage = config_stage.clone();
                                    let error = error.clone();
                                    let touched_fields = touched_fields.clone();
                                    let node_value = node.clone();
                                    let selected = selected_node_name == Some(node.name.as_str());
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
                                            .map(|session| session_state_label(&i18n, &session.state))
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
                                    let onclick = Callback::from(move |_| {
                                        error.set(None);
                                        match node_session_for_click.as_ref().map(|session| &session.state) {
                                            None => {
                                                editing_node.set(None);
                                                draft.set(SessionDraft {
                                                    node: node_value.name.clone(),
                                                    ..SessionDraft::default()
                                                });
                                                touched_fields.set(SessionDraftTouchedControls::new());
                                                config_stage.set(PeerConfigStage::SessionDetails);
                                            }
                                            Some(SessionState::Managed) | Some(SessionState::Manual) | Some(SessionState::StalledPr) => {
                                                let Some(session) = node_session_for_click.as_ref() else {
                                                    error.set(Some(UiMessage::key("error.ui.session.missing_config")));
                                                    return;
                                                };
                                                if session.spec.is_none() {
                                                    error.set(Some(UiMessage::key("error.ui.session.missing_config")));
                                                    return;
                                                }
                                                editing_node.set(Some(node_value.name.clone()));
                                                draft.set(SessionDraft::from_session_view(&node_value.name, session));
                                                touched_fields.set(SessionDraftTouchedControls::new());
                                                config_stage.set(PeerConfigStage::SessionDetails);
                                            }
                                            Some(SessionState::PendingPr) => {
                                                error.set(Some(UiMessage::key("error.ui.operation.wait_inflight")));
                                            }
                                            Some(SessionState::Conflict) => {
                                                error.set(Some(UiMessage::key("error.ui.node.blocked_conflict")));
                                            }
                                        }
                                    });

                                    html! {
                                        <ShellButton
                                            class={classes!(
                                                "autopeer-node-option",
                                                selected.then_some("is-selected"),
                                                (!selectable).then_some("is-unavailable")
                                            )}
                                            onclick={onclick}
                                            disabled={loading || !selectable}
                                        >
                                            <span class="autopeer-node-option-head">
                                                <strong class="autopeer-node-name">{node.name.clone()}</strong>
                                                <span class="autopeer-node-option-status">
                                                    <span class="autopeer-node-badge">{humanize_ip_support(&i18n, &node.ip_support)}</span>
                                                    <span class="autopeer-status-pill">{state_label}</span>
                                                </span>
                                            </span>
                                            <span class="autopeer-node-meta">{node_context_line(&i18n, node)}</span>
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
                        {render_error(&i18n, &error)}
                    </article>
                },
                PeerConfigStage::SessionDetails => html! {
                    <article class="peering-card autopeer-panel">
                        <div class="autopeer-panel-header">
                            <p class="autopeer-panel-kicker">{i18n.t("stage2.kicker")}</p>
                            <h3 class="autopeer-panel-title">
                                {
                                    if let Some(node) = &editing_node_value {
                                        i18n.translate_params(
                                            "stage2.title.update_prefix",
                                            &[("node", node.as_str())],
                                        )
                                    } else if let Some(node) = &selected_node {
                                        i18n.translate_params(
                                            "stage2.title.create_prefix",
                                            &[("node", node.name.as_str())],
                                        )
                                    } else {
                                        i18n.t("stage2.title.create_blank").to_string()
                                    }
                                }
                            </h3>
                            if editing_node_value.is_some() {
                                <p class="text-secondary">
                                    {i18n.t("stage2.update_intro")}
                                </p>
                            }
                        </div>

                        if let Some(node) = &selected_node {
                            <div class="autopeer-node-summary">
                                <div>
                                    <strong>{node_context_line(&i18n, node)}</strong>
                                    if let Some(comment) = &node.comment {
                                        <p class="text-secondary">{comment.clone()}</p>
                                    }
                                </div>
                                <ShellButton text={i18n.t("action.choose_another_node")} onclick={if editing_node_value.is_some() { on_cancel_edit.clone() } else { on_change_node.clone() }} disabled={loading} />
                            </div>
                        }

                        if let Some(session) = selected_session.filter(|s| s.state == SessionState::StalledPr) {
                            <div class="autopeer-stalled-banner">
                                <p class="autopeer-stalled-banner-title">{i18n.t("stalled.banner.title")}</p>
                                <p class="text-secondary">{i18n.t("stalled.banner.body")}</p>
                                <div class="autopeer-links">
                                    if let Some(pr_url) = &session.pull_request_url {
                                        <a href={pr_url.clone()} target="_blank" rel="noreferrer">{i18n.t("action.open_pr")}</a>
                                    }
                                    if let Some(op_id) = &session.pending_operation_id {
                                        <button
                                            class="autopeer-link-button"
                                            onclick={on_retry_operation.reform({
                                                let op_id = op_id.clone();
                                                move |_: MouseEvent| op_id.clone()
                                            })}
                                            disabled={loading}
                                        >
                                            {i18n.t("action.redeploy")}
                                        </button>
                                        <button
                                            class="autopeer-link-button autopeer-link-button--muted"
                                            onclick={on_drop_operation.reform({
                                                let op_id = op_id.clone();
                                                move |_: MouseEvent| op_id.clone()
                                            })}
                                            disabled={loading}
                                        >
                                            {i18n.t("action.drop_changes")}
                                        </button>
                                    }
                                </div>
                            </div>
                        }

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{i18n.t("stage2.section.connection")}</span>
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.endpoint")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.endpoint.clone()}
                                    on_change={update_text_field(|draft| &mut draft.endpoint)}
                                    class={input_class(SessionDraftField::Endpoint)}
                                    frame_class={input_frame_class(SessionDraftField::Endpoint)}
                                    on_focus={on_field_focus(SessionDraftField::Endpoint)}
                                    on_blur={on_field_blur(SessionDraftField::Endpoint)}
                                    placeholder={i18n.t("stage2.field.endpoint.placeholder")}
                                    disabled={loading}
                                />
                                {" "}
                                <ShellToggle
                                    active={draft.encrypt_endpoint && !draft.endpoint.trim().is_empty()}
                                    on_toggle={on_toggle_encrypt_endpoint}
                                    label={i18n.t("stage2.field.encrypt_endpoint")}
                                    disabled={draft.endpoint.trim().is_empty()}
                                />
                            </ShellLine>
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.wg_key")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.wg_public_key.clone()}
                                    on_change={update_text_field(|draft| &mut draft.wg_public_key)}
                                    class={input_class(SessionDraftField::WgPublicKey)}
                                    frame_class={input_frame_class(SessionDraftField::WgPublicKey)}
                                    on_focus={on_field_focus(SessionDraftField::WgPublicKey)}
                                    on_blur={on_field_blur(SessionDraftField::WgPublicKey)}
                                    placeholder={i18n.t("stage2.field.wg_key.placeholder")}
                                    disabled={loading}
                                />
                            </ShellLine>
                            {field_error_message(SessionDraftField::WgPublicKey)}
                        </div>

                        <div class="autopeer-form-section">
                            {live_validation_block(&i18n, live_validation.tunnel_message.as_deref(), html! {
                                <>
                                    <span class="autopeer-section-label">{i18n.t("stage2.section.tunnel")}</span>
                                    <p class="text-secondary">
                                        {i18n.t("stage2.section.tunnel.help")}
                                    </p>
                                    <>
                                        <ShellLine>
                                            <ShellPrompt>{i18n.t("stage2.field.peer4")}</ShellPrompt>
                                            {" "}
                                            <ShellInput
                                                value={draft.peer4.clone()}
                                                on_change={update_text_field(|draft| &mut draft.peer4)}
                                                class={input_class(SessionDraftField::Peer4)}
                                                frame_class={input_frame_class(SessionDraftField::Peer4)}
                                                on_focus={on_field_focus(SessionDraftField::Peer4)}
                                                on_blur={on_field_blur(SessionDraftField::Peer4)}
                                                placeholder={i18n.t("stage2.field.peer4.placeholder")}
                                                disabled={loading}
                                            />
                                        </ShellLine>
                                        {live_validation_message(&i18n, live_validation.peer4_message.as_deref())}
                                    </>
                                    if show_node_ipv4 {
                                        <ShellLine>
                                            <ShellPrompt>{i18n.t("stage2.field.own4_node")}</ShellPrompt>
                                            {" "}
                                            <span class="text-secondary">
                                                {node_inventory_ipv4.clone().unwrap_or_else(|| i18n.t("stage2.field.own4_node.no_inventory").to_string())}
                                            </span>
                                        </ShellLine>
                                    }
                                    <>
                                        <ShellLine>
                                            <ShellPrompt>{i18n.t("stage2.field.peer6")}</ShellPrompt>
                                            {" "}
                                            <ShellInput
                                                value={draft.peer6.clone()}
                                                on_change={on_peer6_change}
                                                class={input_class(SessionDraftField::Peer6)}
                                                frame_class={input_frame_class(SessionDraftField::Peer6)}
                                                on_focus={on_field_focus(SessionDraftField::Peer6)}
                                                on_blur={on_peer6_blur}
                                                placeholder={i18n.t("stage2.field.peer6.placeholder")}
                                                disabled={loading}
                                            />
                                        </ShellLine>
                                        {live_validation_messages(&i18n, &live_validation.peer6_messages)}
                                    </>
                                    if peer6_kind == Some(Peer6AddressKind::LinkLocal) {
                                        <>
                                            <ShellLine>
                                                <ShellPrompt>{i18n.t("stage2.field.own6_link_local")}</ShellPrompt>
                                                {" "}
                                                <ShellInput
                                                    value={draft.own6.clone()}
                                                    on_change={update_text_field(|draft| &mut draft.own6)}
                                                    class={input_class(SessionDraftField::Own6)}
                                                    frame_class={input_frame_class(SessionDraftField::Own6)}
                                                    on_focus={on_field_focus(SessionDraftField::Own6)}
                                                    on_blur={on_field_blur(SessionDraftField::Own6)}
                                                    placeholder={own6_placeholder}
                                                    disabled={loading}
                                                />
                                            </ShellLine>
                                            {live_validation_message(&i18n, live_validation.own6_message.as_deref())}
                                        </>
                                    } else if peer6_kind == Some(Peer6AddressKind::Ula) {
                                        <ShellLine>
                                            <ShellPrompt>{i18n.t("stage2.field.own6_node")}</ShellPrompt>
                                            {" "}
                                            <span class="text-secondary">
                                                {node_inventory_ipv6.clone().unwrap_or_else(|| i18n.t("stage2.field.own6_node.no_inventory").to_string())}
                                            </span>
                                        </ShellLine>
                                    }
                                </>
                            })}
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{i18n.t("stage2.section.families")}</span>
                            <p class="text-secondary">
                                {i18n.t("stage2.section.families.help")}
                            </p>
                            {live_validation_block(&i18n, live_validation.families_message.as_deref(), html! {
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.families")}</ShellPrompt>
                                    {" "}
                                    <span class="autopeer-toggle-row">
                                        <span class={toggle_item_class(live_validation.highlight_ipv4)}>
                                            <ShellToggle
                                                active={draft.ipv4}
                                                on_toggle={on_toggle_ipv4}
                                                label={i18n.t("stage2.field.families.ipv4_label")}
                                            />
                                        </span>
                                        {" "}
                                        <span class={toggle_item_class(live_validation.highlight_ipv6)}>
                                            <ShellToggle
                                                active={draft.ipv6}
                                                on_toggle={on_toggle_ipv6}
                                                label={i18n.t("stage2.field.families.ipv6_label")}
                                            />
                                        </span>
                                    </span>
                                </ShellLine>
                            })}
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{i18n.t("stage2.section.bgp")}</span>
                            <p class="text-secondary">
                                {i18n.t("stage2.section.bgp.help")}
                            </p>
                            {live_validation_block(&i18n, live_validation.bgp_message.as_deref(), html! {
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.bgp_features")}</ShellPrompt>
                                    {" "}
                                    <span class="autopeer-toggle-row">
                                        <span class={toggle_item_class(live_validation.highlight_mp_bgp)}>
                                            <ShellToggle
                                                active={draft.mp_bgp}
                                                on_toggle={on_toggle_mp_bgp}
                                                label={i18n.t("stage2.field.bgp.mpbgp_label")}
                                            />
                                        </span>
                                        {" "}
                                        <span class={toggle_item_class(live_validation.highlight_extended_next_hop)}>
                                            <ShellToggle
                                                active={draft.extended_next_hop}
                                                on_toggle={on_toggle_extended_next_hop}
                                                label={i18n.t("stage2.field.bgp.enh_label")}
                                            />
                                        </span>
                                    </span>
                                </ShellLine>
                            })}
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{i18n.t("stage2.section.policy")}</span>
                            <p class="text-secondary">
                                {peering_strategy_description(&i18n, draft.peering_strategy)}
                            </p>
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.policy")}</ShellPrompt>
                                {" "}
                                <ShellSelect
                                    value={draft.peering_strategy.as_str()}
                                    on_change={on_change_peering_strategy}
                                >
                                    {
                                        for ALL_PEERING_STRATEGIES.iter().map(|strategy| html! {
                                            <option value={strategy.as_str()}>{peering_strategy_label(&i18n, *strategy)}</option>
                                        })
                                    }
                                </ShellSelect>
                            </ShellLine>
                        </div>

                        <details class="autopeer-advanced">
                            <summary>{i18n.t("stage2.advanced.summary")}</summary>
                            <div class="autopeer-form-section autopeer-form-section--advanced">
                                if draft.mp_bgp {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("stage2.field.bgp.transport")}</ShellPrompt>
                                        {" "}
                                        <ShellSelect
                                            value={draft.selected_mp_bgp_transport().as_str()}
                                            on_change={on_change_mp_bgp_transport}
                                        >
                                            {
                                                for ALL_MP_BGP_TRANSPORTS.iter().map(|transport| html! {
                                                    <option value={transport.as_str()}>{mp_bgp_transport_label(&i18n, *transport)}</option>
                                                })
                                            }
                                        </ShellSelect>
                                    </ShellLine>
                                }
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.comment")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.comment.clone()}
                                        on_change={update_text_field(|draft| &mut draft.comment)}
                                        placeholder={i18n.t("stage2.field.comment.placeholder")}
                                        disabled={loading}
                                    />
                                </ShellLine>
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.keepalive")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.keepalive.clone()}
                                        on_change={update_text_field(|draft| &mut draft.keepalive)}
                                        class={input_class(SessionDraftField::Keepalive)}
                                        frame_class={input_frame_class(SessionDraftField::Keepalive)}
                                        on_focus={on_field_focus(SessionDraftField::Keepalive)}
                                        on_blur={on_field_blur(SessionDraftField::Keepalive)}
                                        placeholder={i18n.t("stage2.field.keepalive.placeholder")}
                                        disabled={loading}
                                    />
                                </ShellLine>
                                {field_error_message(SessionDraftField::Keepalive)}
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.mtu")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.mtu.clone()}
                                        on_change={update_text_field(|draft| &mut draft.mtu)}
                                        class={input_class(SessionDraftField::Mtu)}
                                        frame_class={input_frame_class(SessionDraftField::Mtu)}
                                        on_focus={on_field_focus(SessionDraftField::Mtu)}
                                        on_blur={on_field_blur(SessionDraftField::Mtu)}
                                        placeholder={i18n.t("stage2.field.mtu.placeholder")}
                                        disabled={loading}
                                    />
                                </ShellLine>
                                {field_error_message(SessionDraftField::Mtu)}
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.psk")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.psk.clone()}
                                        on_change={update_text_field(|draft| &mut draft.psk)}
                                        class={input_class(SessionDraftField::Psk)}
                                        frame_class={input_frame_class(SessionDraftField::Psk)}
                                        on_focus={on_field_focus(SessionDraftField::Psk)}
                                        on_blur={on_field_blur(SessionDraftField::Psk)}
                                        placeholder={if draft.has_psk { i18n.t("stage2.field.psk.placeholder.existing") } else { i18n.t("stage2.field.psk.placeholder") }}
                                        disabled={loading}
                                    />
                                    {" "}
                                    <ShellButton
                                        text={if *psk_copied {
                                            i18n.t("stage2.field.psk.copied")
                                        } else if draft.has_psk || !draft.psk.is_empty() {
                                            i18n.t("stage2.field.psk.clear")
                                        } else {
                                            i18n.t("stage2.field.psk.generate")
                                        }}
                                        onclick={on_psk_action.clone()}
                                        disabled={loading || *psk_copied}
                                    />
                                </ShellLine>
                                {field_error_message(SessionDraftField::Psk)}
                            </div>
                        </details>

                        {render_ongoing_tasks(&i18n, ongoing_tasks.tasks())}
                        {render_error(&i18n, &error)}

                        <div class="autopeer-inline-actions">
                            if editing_node_value.is_some() {
                                <ShellButton text={i18n.t("action.cancel_edit")} onclick={on_cancel_edit.clone()} disabled={loading} />
                                <ShellButton
                                    text={retire_button_text(&i18n, retire_confirmation_value)}
                                    onclick={on_retire_selected_session.clone()}
                                    disabled={loading}
                                />
                                <ShellButton
                                    text={delete_button_text(&i18n, delete_confirmation_value)}
                                    onclick={on_delete_selected_session.clone()}
                                    disabled={loading}
                                />
                            } else {
                                <ShellButton text={i18n.t("action.back_to_nodes")} onclick={on_change_node.clone()} disabled={loading} />
                            }
                            <ShellButton
                                text={if editing_node_value.is_some() { i18n.t("action.review_your_update") } else { i18n.t("action.review_your_change") }}
                                onclick={on_continue_to_review}
                                disabled={
                                    loading
                                        || (editing_node_value.is_none() && draft.node.trim().is_empty())
                                        || !draft_is_valid
                                }
                            />
                        </div>
                    </article>
                },
                PeerConfigStage::Review => html! {
                    <article class="peering-card autopeer-panel">
                        <div class="autopeer-panel-header">
                            <p class="autopeer-panel-kicker">{i18n.t("stage3.kicker")}</p>
                            <h3 class="autopeer-panel-title">{i18n.t("stage3.title")}</h3>
                        </div>

                        <div class="autopeer-review-grid">
                            {review_item(
                                i18n.t("stage3.review.our_node"),
                                selected_node
                                    .as_ref()
                                    .map(|node| node_review_line(&i18n, node))
                                    .unwrap_or_else(|| i18n.t("stage3.review.not_selected").to_string()),
                            )}
                            {review_item(i18n.t("stage3.review.endpoint"), draft.endpoint.clone())}
                            {review_item(i18n.t("stage3.review.wg_key"), draft.wg_public_key.clone())}
                            {review_item(
                                i18n.t("stage3.review.route_families"),
                                i18n.t(draft.families_label_key()).to_string(),
                            )}
                            {review_item(
                                i18n.t("stage3.review.bgp_behavior"),
                                format!(
                                    "{}{}{}",
                                    if draft.mp_bgp { i18n.t("stage3.review.bgp.mpbgp") } else { i18n.t("stage3.review.bgp.separate") },
                                    if draft.mp_bgp {
                                        format!(
                                            " ({})",
                                            mp_bgp_transport_label(&i18n, draft.selected_mp_bgp_transport())
                                        )
                                    } else {
                                        String::new()
                                    },
                                    if draft.extended_next_hop { i18n.t("stage3.review.bgp.enh_suffix") } else { "" },
                                ),
                            )}
                            {review_item(
                                i18n.t("stage3.review.routing_policy"),
                                peering_strategy_label(&i18n, draft.peering_strategy).to_string(),
                            )}
                            {optional_review_item(i18n.t("stage3.review.peer4"), &draft.peer4)}
                            {optional_review_item(i18n.t("stage3.review.peer6"), &draft.peer6)}
                            {optional_review_item(i18n.t("stage3.review.own6"), &draft.own6)}
                            {optional_review_item(i18n.t("stage3.review.keepalive"), &draft.keepalive)}
                            {optional_review_item(i18n.t("stage3.review.mtu"), &draft.mtu)}
                            {review_item(
                                i18n.t("stage3.review.psk"),
                                if !draft.psk.trim().is_empty() {
                                    i18n.t("stage3.review.psk.set").to_string()
                                } else if draft.clear_psk {
                                    i18n.t("stage3.review.psk.cleared").to_string()
                                } else if draft.has_psk {
                                    i18n.t("stage3.review.psk.unchanged").to_string()
                                } else {
                                    i18n.t("stage3.review.psk.not_set").to_string()
                                },
                            )}
                            {review_item(
                                i18n.t("stage3.review.encrypt_endpoint"),
                                if draft.encrypt_endpoint {
                                    i18n.t("stage3.review.encrypt_endpoint.enabled").to_string()
                                } else {
                                    i18n.t("stage3.review.encrypt_endpoint.disabled").to_string()
                                },
                            )}
                            {optional_review_item(i18n.t("stage3.review.note"), &draft.comment)}
                        </div>

                        {render_inventory_peering_review(&i18n, selected_node.as_ref(), &active_asn)}

                        {render_ongoing_tasks(&i18n, ongoing_tasks.tasks())}
                        {render_error(&i18n, &error)}

                        <div class="autopeer-inline-actions">
                            <ShellButton text={i18n.t("action.back_to_details")} onclick={on_back_to_details} disabled={loading} />
                            if editing_node_value.is_some() {
                                <ShellButton text={i18n.t("action.cancel_edit")} onclick={on_cancel_edit} disabled={loading} />
                            } else {
                                <ShellButton text={i18n.t("action.choose_another_node")} onclick={on_change_node} disabled={loading} />
                            }
                            <ShellButton
                                text={if editing_node_value.is_some() { i18n.t("action.open_update_pr") } else { i18n.t("action.open_create_pr") }}
                                onclick={on_submit_session}
                                disabled={
                                    loading
                                        || (editing_node_value.is_none() && draft.node.trim().is_empty())
                                        || !draft_is_valid
                                }
                            />
                        </div>
                    </article>
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
                            <ShellButton text={i18n.t("action.refresh")} onclick={on_refresh.clone()} disabled={loading} />
                            <ShellButton text={i18n.t("action.logout")} onclick={on_logout} disabled={loading} />
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
                                    {render_flow_steps(&i18n, active_stage, &on_step_click)}
                                    {main_panel}
                                </>
                            }
                        </div>

                        <aside class="autopeer-sidebar">
                            <article class="peering-card autopeer-panel autopeer-panel--compact">
                                <div class="autopeer-panel-header">
                                    <p class="autopeer-panel-kicker">{i18n.t("sidebar.your_session_kicker")}</p>
                                    <h3 class="autopeer-panel-title">
                                        {auth_summary.as_ref().map(|session| format!("AS{}", session.asn)).unwrap_or_else(|| i18n.t("sidebar.no_active_session").to_string())}
                                    </h3>
                                    {auth_summary.as_ref().map(|session| {
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

                            {host_summary.as_ref().map(|host_session| {
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
                                                        value={(*impersonate_asn).clone()}
                                                        on_change={on_impersonate_asn_change}
                                                        placeholder={i18n.t("sidebar.impersonate_asn_placeholder")}
                                                        disabled={loading}
                                                    />
                                                </ShellLine>
                                                <ShellLine>
                                                    <ShellPrompt>{i18n.t("sidebar.effective_mnt_label")}</ShellPrompt>
                                                    {" "}
                                                    <ShellInput
                                                        value={(*impersonate_mnt).clone()}
                                                        on_change={on_impersonate_mnt_change}
                                                        placeholder={i18n.t("sidebar.impersonate_mnt_placeholder")}
                                                        disabled={loading}
                                                    />
                                                </ShellLine>
                                                {render_error(&i18n, &support_error)}
                                                <div class="autopeer-inline-actions">
                                                    <ShellButton
                                                        text={i18n.t("action.impersonate_this_asn")}
                                                        onclick={on_impersonate}
                                                        disabled={loading || impersonate_asn.trim().is_empty()}
                                                    />
                                                    if auth_summary.as_ref().map(|session| session.asn.as_str()) != Some(host_session.asn.as_str()) {
                                                        <ShellButton
                                                            text={i18n.t("action.return_to_host_asn")}
                                                            onclick={on_return_to_host}
                                                            disabled={loading}
                                                        />
                                                    }
                                                </div>
                                            </div>
                                        </article>
                                    }
                            }).unwrap_or_default()}

                            if let Some(operation_status) = &*operation {
                                <article class="peering-card autopeer-panel autopeer-panel--compact autopeer-status-card">
                                    <div class="autopeer-panel-header">
                                        <p class="autopeer-panel-kicker">{i18n.t("sidebar.current_operation")}</p>
                                        <h3 class="autopeer-panel-title">
                                            {format!("{} {}", operation_kind_label(&i18n, &operation_status.kind), operation_status.node)}
                                        </h3>
                                        <span class="autopeer-status-pill">{operation_state_label(&i18n, &operation_status.state)}</span>
                                        if operation_status.failure_details.is_none() {
                                            if let Some(message) = &operation_status.message {
                                                <p class="text-secondary">{i18n.translate_message(message)}</p>
                                            }
                                        }
                                    </div>
                                    {render_operation_progress(&i18n, operation_status)}
                                    if let Some(details) = &operation_status.failure_details {
                                        <div class="autopeer-failure-details">
                                            <p class="autopeer-failure-stage">
                                                <strong>{i18n.t("operation.failure.stage")}{": "}</strong>
                                                {operation_failure_stage_label(&i18n, &details.stage)}
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
                                                    onclick={on_retry_operation.reform({
                                                        let id = operation_status.id.clone();
                                                        move |_: MouseEvent| id.clone()
                                                    })}
                                                    disabled={loading}
                                                >
                                                    {i18n.t("action.retry")}
                                                </button>
                                            }
                                            <button
                                                class="autopeer-link-button autopeer-link-button--muted"
                                                onclick={on_dismiss_operation.clone()}
                                            >
                                                {i18n.t("action.dismiss_operation")}
                                            </button>
                                        }
                                    </div>
                                </article>
                            }
                        </aside>
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
                            {" / "}
                            <a href={(*looking_glass_site_href).clone()} class="autopeer-title-nav">{i18n.t("nav.looking_glass")}</a>
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

    use super::{
        autopeer_node_endpoint_port, delete_button_text, displayed_peer_config_stage,
        retire_button_text,
    };
    use crate::{
        models::{MpBgpTransport, PeeringStrategy},
        store::{PeerConfigStage, SessionDraft, SessionDraftField},
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
    fn keeps_editing_sessions_on_stage_two_until_edit_is_cleared() {
        assert_eq!(
            displayed_peer_config_stage(Some("hkg"), PeerConfigStage::SelectNode),
            PeerConfigStage::SessionDetails
        );
        assert_eq!(
            displayed_peer_config_stage(None, PeerConfigStage::SelectNode),
            PeerConfigStage::SelectNode
        );
    }

    #[test]
    fn retire_button_requires_confirmation_click() {
        let i18n = crate::i18n::I18n::test_default();
        assert_eq!(retire_button_text(&i18n, false), "Retire This Session");
        assert_eq!(retire_button_text(&i18n, true), "Confirm Retirement");
    }

    #[test]
    fn delete_button_requires_confirmation_click() {
        let i18n = crate::i18n::I18n::test_default();
        assert_eq!(delete_button_text(&i18n, false), "Delete This Session");
        assert_eq!(delete_button_text(&i18n, true), "Confirm Deletion");
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
        let touched = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

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

        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.required_ipv6".to_string()]
        );
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
    fn live_validation_places_ipv6_route_requirement_on_peer6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            mp_bgp: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([SessionDraftToggleGroup::Families.into()]);

        let validation = session_details_live_validation(&draft, &touched, None, None);

        assert_eq!(
            validation.peer6_messages,
            vec!["validation.peer6.required_ipv6".to_string()]
        );
        assert!(!validation.highlight_peer6);
        assert!(validation.highlight_ipv6);
        assert!(!validation.highlight_extended_next_hop);
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
    fn live_validation_hides_missing_peer6_message_while_peer6_is_focused() {
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
    fn live_validation_shows_missing_peer6_message_while_focused_after_clear() {
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
