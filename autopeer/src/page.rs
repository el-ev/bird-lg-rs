use std::collections::BTreeSet;

use common::{
    auto_peer::{
        ALL_PEERING_STRATEGIES, AuthMethodKind, NodeView, OperationFailureStage, OperationKind,
        OperationState, OperationStatus, PeeringStrategy, SessionState, SessionView,
    },
    models::PeeringInfo,
};
use ui_components::shell::{
    ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect, ShellToggle,
};
use web_sys::{HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::{
    controller::{
        AutoPeerController, OngoingTask, default_pgp_key, selected_registry_email_target,
        sync_create_draft, use_autopeer_controller,
    },
    i18n::{I18n, use_i18n},
    store::{AutoPeerStep, PeerConfigStage, SessionDraft, SessionDraftField},
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

fn field_key(field: SessionDraftField) -> &'static str {
    match field {
        SessionDraftField::Endpoint => "endpoint",
        SessionDraftField::WgPublicKey => "wg_public_key",
        SessionDraftField::Peer4 => "peer4",
        SessionDraftField::Peer6 => "peer6",
        SessionDraftField::Own6 => "own6",
        SessionDraftField::Keepalive => "keepalive",
        SessionDraftField::Mtu => "mtu",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionDraftToggleGroup {
    Families,
    Bgp,
}

fn toggle_group_key(group: SessionDraftToggleGroup) -> &'static str {
    match group {
        SessionDraftToggleGroup::Families => "__toggle_group.families",
        SessionDraftToggleGroup::Bgp => "__toggle_group.bgp",
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

fn mark_field_touched(touched_fields: &UseStateHandle<BTreeSet<String>>, field: SessionDraftField) {
    let mut next = (**touched_fields).clone();
    next.insert(field_key(field).to_string());
    touched_fields.set(next);
}

fn mark_toggle_group_touched(
    touched_fields: &UseStateHandle<BTreeSet<String>>,
    group: SessionDraftToggleGroup,
) {
    let mut next = (**touched_fields).clone();
    next.insert(toggle_group_key(group).to_string());
    touched_fields.set(next);
}

fn control_is_touched(touched_controls: &BTreeSet<String>, field: SessionDraftField) -> bool {
    touched_controls.contains(field_key(field))
}

fn toggle_group_is_touched(
    touched_controls: &BTreeSet<String>,
    group: SessionDraftToggleGroup,
) -> bool {
    touched_controls.contains(toggle_group_key(group))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SessionDraftLiveValidation {
    tunnel_message: Option<String>,
    families_message: Option<String>,
    bgp_message: Option<String>,
    highlight_peer4: bool,
    highlight_peer6: bool,
    highlight_own6: bool,
    highlight_ipv4: bool,
    highlight_ipv6: bool,
    highlight_mp_bgp: bool,
}

impl SessionDraftLiveValidation {
    fn highlights_field(&self, field: SessionDraftField) -> bool {
        match field {
            SessionDraftField::Peer4 => self.highlight_peer4,
            SessionDraftField::Peer6 => self.highlight_peer6,
            SessionDraftField::Own6 => self.highlight_own6,
            _ => false,
        }
    }
}

fn session_details_live_validation(
    draft: &SessionDraft,
    touched_controls: &BTreeSet<String>,
) -> SessionDraftLiveValidation {
    let peer4_touched = control_is_touched(touched_controls, SessionDraftField::Peer4);
    let peer6_touched = control_is_touched(touched_controls, SessionDraftField::Peer6);
    let own6_touched = control_is_touched(touched_controls, SessionDraftField::Own6);
    let families_touched =
        toggle_group_is_touched(touched_controls, SessionDraftToggleGroup::Families);
    let bgp_touched = toggle_group_is_touched(touched_controls, SessionDraftToggleGroup::Bgp);
    let combo_touched = families_touched || bgp_touched;

    let peer4_blank = draft.peer4.trim().is_empty();
    let peer6_blank = draft.peer6.trim().is_empty();
    let own6_present = !draft.own6.trim().is_empty();

    let no_families_selected = !draft.ipv4 && !draft.ipv6;
    let peer4_missing = draft.ipv4 && !draft.mp_bgp && peer4_blank;
    let peer6_missing_for_ipv6 = draft.ipv6 && peer6_blank;
    let peer6_missing_for_mp_bgp = draft.mp_bgp && peer6_blank;

    let own6_missing_peer6 = own6_present && peer6_blank;
    let own6_requires_link_local_peer6 =
        own6_present && !peer6_blank && !draft.peer6_is_link_local();
    let own6_message = if own6_present || own6_touched {
        draft.field_error(SessionDraftField::Own6)
    } else {
        None
    };

    SessionDraftLiveValidation {
        tunnel_message: if (peer4_touched || peer6_touched || own6_touched)
            && peer4_blank
            && peer6_blank
        {
            Some("validation.tunnel.required".to_string())
        } else {
            own6_message.clone()
        },
        families_message: if families_touched && no_families_selected {
            Some("validation.bgp_family.required".to_string())
        } else {
            None
        },
        bgp_message: if !combo_touched {
            None
        } else if peer6_missing_for_mp_bgp {
            draft.field_error(SessionDraftField::Peer6)
        } else if peer4_missing {
            draft.field_error(SessionDraftField::Peer4)
        } else if peer6_missing_for_ipv6 {
            draft.field_error(SessionDraftField::Peer6)
        } else {
            None
        },
        highlight_peer4: (peer4_touched && draft.field_error(SessionDraftField::Peer4).is_some())
            || (peer4_missing && combo_touched),
        highlight_peer6: (peer6_touched && draft.field_error(SessionDraftField::Peer6).is_some())
            || ((peer6_missing_for_mp_bgp || peer6_missing_for_ipv6) && combo_touched)
            || own6_missing_peer6
            || own6_requires_link_local_peer6,
        highlight_own6: own6_message.is_some(),
        highlight_ipv4: (families_touched && no_families_selected)
            || (peer4_missing && combo_touched),
        highlight_ipv6: (families_touched && no_families_selected)
            || (peer6_missing_for_ipv6 && combo_touched),
        highlight_mp_bgp: peer6_missing_for_mp_bgp && combo_touched,
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

fn render_error(i18n: &I18n, error: &Option<String>) -> Html {
    if let Some(error) = error {
        html! {
            <ShellLine>
                <span class="error-message">{i18n.translate_owned(error)}</span>
            </ShellLine>
        }
    } else {
        Html::default()
    }
}

fn render_loading_message(i18n: &I18n, raw: &str) -> String {
    i18n.translate_owned(raw)
}

fn render_loading(i18n: &I18n, loading: bool, loading_message: Option<&str>) -> Html {
    if !loading {
        return Html::default();
    }
    let message = match loading_message {
        Some(raw) => render_loading_message(i18n, raw),
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
                let message = render_loading_message(i18n, &task.message);
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

fn autopeer_home_href() -> String {
    "/".to_string()
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

fn session_for_node<'a>(node_name: &str, sessions: &'a [SessionView]) -> Option<&'a SessionView> {
    sessions.iter().find(|session| session.node == node_name)
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

fn node_context_line(i18n: &I18n, node: &NodeView) -> String {
    let mut parts = Vec::new();

    if let Some(region) = humanize_region(i18n, &node.region) {
        parts.push(region);
    }
    if let Some(country) = &node.country {
        parts.push(country.to_string());
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
        SessionState::Conflict => i18n.t("session_state.conflict"),
    }
}

fn operation_kind_label(i18n: &I18n, kind: &OperationKind) -> &'static str {
    match kind {
        OperationKind::Create => i18n.t("operation.kind.create"),
        OperationKind::Update => i18n.t("operation.kind.update"),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Peer6AddressKind {
    LinkLocal,
    Ula,
}

fn detect_peer6_address_kind(value: &str) -> Option<Peer6AddressKind> {
    let trimmed = value.trim().to_ascii_lowercase();
    if trimmed.starts_with("fe80:") {
        Some(Peer6AddressKind::LinkLocal)
    } else if trimmed.starts_with("fd") || trimmed.starts_with("fc") {
        Some(Peer6AddressKind::Ula)
    } else {
        None
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
                common::auto_peer::OperationFailureStage::Checks => 1,
                common::auto_peer::OperationFailureStage::Preflight
                | common::auto_peer::OperationFailureStage::Apply => 2,
                common::auto_peer::OperationFailureStage::Merge => 3,
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

fn render_flow_steps(i18n: &I18n, stage: PeerConfigStage) -> Html {
    let steps = [
        PeerConfigStage::SelectNode,
        PeerConfigStage::SessionDetails,
        PeerConfigStage::Review,
    ];

    html! {
        <ol class="autopeer-flow-steps">
            {for steps.into_iter().map(|candidate| {
                let state_class = if candidate == stage {
                    "is-active"
                } else if candidate.index() < stage.index() {
                    "is-complete"
                } else {
                    "is-upcoming"
                };
                html! {
                    <li class={classes!("autopeer-flow-step", state_class)}>
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
        common::auto_peer::OperationState::Failed | common::auto_peer::OperationState::Conflict
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
    let default_autopeer_home_href = autopeer_home_href();
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
    } = use_autopeer_controller(default_autopeer_home_href, default_looking_glass_href);
    let loading = !ongoing_tasks.is_empty();

    let content = match &*step {
        AutoPeerStep::LoadingConfig => html! {
            <div class="autopeer-step">
                <ShellLine>
                    <ShellPrompt>{i18n.t("prompt.autopeer")}</ShellPrompt>
                    {" "}{i18n.t("step.loading_config.prompt")}
                </ShellLine>
                {render_loading(&i18n, true, Some(i18n.t("step.loading_config.message")))}
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
                                let onclick = Callback::from(move |_| {
                                    on_enter_oidc.emit(method_copy.clone());
                                });

                                html! {
                                    <ShellLine>
                                        <ShellButton
                                            text={format!("{}{}", i18n.t("step.enter_asn.continue_with"), method.label)}
                                            onclick={onclick}
                                            disabled={loading}
                                        />
                                        <span class="autopeer-method-desc">
                                            {format!(" - {}", method.description)}
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
                        {format!(" {}{}", i18n.t("step.select_method.found_for_as"), *asn)}
                    </ShellLine>
                    <div class="autopeer-challenge-list">
                        {for methods.iter().map(|method| {
                            let on_select_method = on_select_method.clone();
                            let method_value = method.clone();
                            let onclick = Callback::from(move |_| {
                                on_select_method.emit(method_value.clone());
                            });

                            html! {
                                <ShellLine>
                                    <ShellButton
                                        text={method.label.clone()}
                                        onclick={onclick}
                                        disabled={loading}
                                    />
                                    <span class="autopeer-method-desc">
                                        {format!(" - {}", method.description)}
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
                                        {format!(" {}{}", i18n.t("verify.ssh.match_one"), method.ssh_fingerprints[0])}
                                    </ShellLine>
                                } else {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.keys")}</ShellPrompt>
                                        {format!(
                                            " {}{}",
                                            i18n.t("verify.ssh.match_many"),
                                            method.ssh_fingerprints.join(", ")
                                        )}
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
                                        {format!(" {}{}", i18n.t("verify.pgp.use_key"), method.pgp_fingerprints[0])}
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
                                        {format!(" {}{}", i18n.t("verify.email.auth_as"), method.email_targets[0].maintainer)}
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
                                if let Some(target) = selected_target {
                                    <ShellLine>
                                        <ShellPrompt>{i18n.t("prompt.emails")}</ShellPrompt>
                                        {format!(" {}{}", i18n.t("verify.email.send_to"), target.emails.join(", "))}
                                    </ShellLine>
                                }
                                if !registry_email_sent_to.is_empty() {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {format!(
                                                "{}{}.",
                                                i18n.t("verify.email.sent_to_prefix"),
                                                registry_email_sent_to.join(", ")
                                            )}
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
                                    {format!(" {}{}{}", i18n.t("verify.oidc.continue_to"), method.label, i18n.t("verify.oidc.in_browser"))}
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
                    format!("{}{}", i18n.t("verify.oidc.continue_to"), method.label)
                } else if method.kind == AuthMethodKind::RegistryEmail {
                    i18n.t("action.verify_code").to_string()
                } else {
                    i18n.t("action.verify").to_string()
                };

                html! {
                    <div class="autopeer-step">
                        <ShellLine>
                            <ShellPrompt>{i18n.t("prompt.auth")}</ShellPrompt>
                            {format!(" {}{}{}", method.label, i18n.t("verify.auth_for_as"), *asn)}
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
            let retire_confirmation_value = *retire_confirmation;
            let active_asn = auth_summary
                .as_ref()
                .map(|session| session.asn.clone())
                .unwrap_or_else(|| (*asn).clone());
            let draft_is_valid = draft.to_spec().is_ok();
            let live_validation = session_details_live_validation(&draft, &touched_fields);
            let peer6_kind = detect_peer6_address_kind(&draft.peer6);
            let node_inventory_ipv6 = selected_node
                .as_ref()
                .and_then(|node| node.peering.as_ref())
                .and_then(|peering| peering.ipv6.clone());
            let node_inventory_link_local_ipv6 = selected_node
                .as_ref()
                .and_then(|node| node.peering.as_ref())
                .and_then(|peering| peering.link_local_ipv6.clone());
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
                    touched_fields.set(BTreeSet::new());
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
                Callback::from(move |_: FocusEvent| mark_field_touched(&touched_fields, field))
            };

            let field_is_invalid = |field: SessionDraftField| {
                control_is_touched(&touched_fields, field) && draft.field_error(field).is_some()
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

            let toggle_item_class =
                |invalid: bool| classes!("autopeer-toggle-item", invalid.then_some("is-invalid"));

            let on_peer6_change = {
                let draft = draft.clone();
                Callback::from(move |value: String| {
                    update_draft_state(&draft, |next| {
                        next.peer6 = value;
                        if !next.peer6_is_link_local() {
                            next.own6.clear();
                        }
                    });
                })
            };

            let on_toggle_ipv4 = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    mark_toggle_group_touched(&touched_fields, SessionDraftToggleGroup::Families);
                    update_draft_state(&draft, |next| next.ipv4 = !next.ipv4);
                })
            };

            let on_toggle_ipv6 = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    mark_toggle_group_touched(&touched_fields, SessionDraftToggleGroup::Families);
                    update_draft_state(&draft, |next| next.ipv6 = !next.ipv6);
                })
            };

            let on_toggle_mp_bgp = {
                let draft = draft.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_: ()| {
                    mark_toggle_group_touched(&touched_fields, SessionDraftToggleGroup::Bgp);
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
                    mark_toggle_group_touched(&touched_fields, SessionDraftToggleGroup::Bgp);
                    update_draft_state(&draft, |next| {
                        next.extended_next_hop = !next.extended_next_hop;
                        if next.extended_next_hop {
                            next.mp_bgp = true;
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

            let on_back_to_details = {
                let config_stage = config_stage.clone();
                Callback::from(move |_| config_stage.set(PeerConfigStage::SessionDetails))
            };

            let on_change_node = {
                let config_stage = config_stage.clone();
                Callback::from(move |_| config_stage.set(PeerConfigStage::SelectNode))
            };

            let on_continue_to_review = {
                let draft = draft.clone();
                let editing_node = editing_node.clone();
                let config_stage = config_stage.clone();
                let error = error.clone();
                let i18n = i18n.clone();
                Callback::from(move |_| {
                    if editing_node.is_none() && draft.node.trim().is_empty() {
                        error.set(Some(i18n.t("error.choose_node").to_string()));
                        return;
                    }

                    match draft.to_spec() {
                        Ok(_) => {
                            error.set(None);
                            config_stage.set(PeerConfigStage::Review);
                        }
                        Err(message) => error.set(Some(message)),
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
                                    let node_session = session_for_node(&node.name, &sessions).cloned();
                                    let node_session_for_click = node_session.clone();
                                    let draft = draft.clone();
                                    let editing_node = editing_node.clone();
                                    let config_stage = config_stage.clone();
                                    let error = error.clone();
                                    let node_value = node.clone();
                                    let selected = selected_node_name == Some(node.name.as_str());
                                    let autopeer_disabled = node.autopeer == Some(false);
                                    let selectable = !autopeer_disabled && matches!(
                                        node_session.as_ref().map(|session| &session.state),
                                        None | Some(SessionState::Managed) | Some(SessionState::Manual)
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
                                            Some(SessionState::Conflict) => i18n.t("stage1.state.note.conflict"),
                                        }
                                    };
                                    let i18n_for_click = i18n.clone();
                                    let onclick = Callback::from(move |_| {
                                        error.set(None);
                                        match node_session_for_click.as_ref().map(|session| &session.state) {
                                            None => {
                                                editing_node.set(None);
                                                update_draft_state(&draft, |next| {
                                                    next.node = node_value.name.clone();
                                                    next.peering_strategy = PeeringStrategy::FullTable;
                                                });
                                                config_stage.set(PeerConfigStage::SessionDetails);
                                            }
                                            Some(SessionState::Managed) | Some(SessionState::Manual) => {
                                                let Some(spec) = node_session_for_click.as_ref().and_then(|session| session.spec.clone()) else {
                                                    error.set(Some(i18n_for_click.t("error.session_missing_config").to_string()));
                                                    return;
                                                };
                                                editing_node.set(Some(node_value.name.clone()));
                                                draft.set(SessionDraft::from_session(&node_value.name, &spec));
                                                config_stage.set(PeerConfigStage::SessionDetails);
                                            }
                                            Some(SessionState::PendingPr) => {
                                                error.set(Some(i18n_for_click.t("error.wait_inflight").to_string()));
                                            }
                                            Some(SessionState::Conflict) => {
                                                error.set(Some(i18n_for_click.t("error.node_blocked_conflict").to_string()));
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
                                                    <span class="autopeer-node-badge">{node.ip_support.clone()}</span>
                                                    <span class="autopeer-status-pill">{state_label}</span>
                                                </span>
                                            </span>
                                            <span class="autopeer-node-meta">{node_context_line(&i18n, node)}</span>
                                            if let Some(comment) = &node.comment {
                                                <span class="autopeer-node-note">{comment.clone()}</span>
                                            }
                                            if let Some(message) = node_session.as_ref().and_then(|session| session.message.as_ref()) {
                                                <span class="autopeer-node-note">{message.clone()}</span>
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
                                        format!("{}{}", i18n.t("stage2.title.update_prefix"), node)
                                    } else if let Some(node) = &selected_node {
                                        format!("{}{}", i18n.t("stage2.title.create_prefix"), node.name)
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
                                if editing_node_value.is_none() {
                                    <ShellButton text={i18n.t("action.choose_another_node")} onclick={on_change_node.clone()} disabled={loading} />
                                }
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
                                    on_blur={on_field_blur(SessionDraftField::Endpoint)}
                                    placeholder={i18n.t("stage2.field.endpoint.placeholder")}
                                    disabled={loading}
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
                                    on_blur={on_field_blur(SessionDraftField::WgPublicKey)}
                                    placeholder={i18n.t("stage2.field.wg_key.placeholder")}
                                    disabled={loading}
                                />
                            </ShellLine>
                        </div>

                        <div class={classes!(
                            "autopeer-form-section",
                            live_validation
                                .tunnel_message
                                .is_some()
                                .then_some("autopeer-form-section--invalid")
                        )}>
                            <span class="autopeer-section-label">{i18n.t("stage2.section.tunnel")}</span>
                            <p class="text-secondary">
                                {i18n.t("stage2.section.tunnel.help")}
                            </p>
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.peer4")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.peer4.clone()}
                                    on_change={update_text_field(|draft| &mut draft.peer4)}
                                    class={input_class(SessionDraftField::Peer4)}
                                    frame_class={input_frame_class(SessionDraftField::Peer4)}
                                    on_blur={on_field_blur(SessionDraftField::Peer4)}
                                    placeholder={i18n.t("stage2.field.peer4.placeholder")}
                                    disabled={loading}
                                />
                            </ShellLine>
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.peer6")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.peer6.clone()}
                                    on_change={on_peer6_change}
                                    class={input_class(SessionDraftField::Peer6)}
                                    frame_class={input_frame_class(SessionDraftField::Peer6)}
                                    on_blur={on_field_blur(SessionDraftField::Peer6)}
                                    placeholder={i18n.t("stage2.field.peer6.placeholder")}
                                    disabled={loading}
                                />
                            </ShellLine>
                            if peer6_kind == Some(Peer6AddressKind::LinkLocal) {
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.own6_link_local")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.own6.clone()}
                                        on_change={update_text_field(|draft| &mut draft.own6)}
                                        class={input_class(SessionDraftField::Own6)}
                                        frame_class={input_frame_class(SessionDraftField::Own6)}
                                        on_blur={on_field_blur(SessionDraftField::Own6)}
                                        placeholder={own6_placeholder}
                                        disabled={loading}
                                    />
                                </ShellLine>
                            } else if peer6_kind == Some(Peer6AddressKind::Ula) {
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.own6_node")}</ShellPrompt>
                                    {" "}
                                    <span class="text-secondary">
                                        {node_inventory_ipv6.clone().unwrap_or_else(|| i18n.t("stage2.field.own6_node.no_inventory").to_string())}
                                    </span>
                                </ShellLine>
                            }
                            {live_validation_message(&i18n, live_validation.tunnel_message.as_deref())}
                        </div>

                        <div class={classes!(
                            "autopeer-form-section",
                            live_validation
                                .families_message
                                .is_some()
                                .then_some("autopeer-form-section--invalid")
                        )}>
                            <span class="autopeer-section-label">{i18n.t("stage2.section.families")}</span>
                            <p class="text-secondary">
                                {i18n.t("stage2.section.families.help")}
                            </p>
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
                            {live_validation_message(&i18n, live_validation.families_message.as_deref())}
                        </div>

                        <div class={classes!(
                            "autopeer-form-section",
                            live_validation
                                .bgp_message
                                .is_some()
                                .then_some("autopeer-form-section--invalid")
                        )}>
                            <span class="autopeer-section-label">{i18n.t("stage2.section.bgp")}</span>
                            <p class="text-secondary">
                                {i18n.t("stage2.section.bgp.help")}
                            </p>
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
                                    <span class={toggle_item_class(false)}>
                                        <ShellToggle
                                            active={draft.extended_next_hop}
                                            on_toggle={on_toggle_extended_next_hop}
                                            label={i18n.t("stage2.field.bgp.enh_label")}
                                        />
                                    </span>
                                </span>
                            </ShellLine>
                            {live_validation_message(&i18n, live_validation.bgp_message.as_deref())}
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
                                        on_blur={on_field_blur(SessionDraftField::Keepalive)}
                                        placeholder={i18n.t("stage2.field.keepalive.placeholder")}
                                        disabled={loading}
                                    />
                                </ShellLine>
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.mtu")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.mtu.clone()}
                                        on_change={update_text_field(|draft| &mut draft.mtu)}
                                        class={input_class(SessionDraftField::Mtu)}
                                        frame_class={input_frame_class(SessionDraftField::Mtu)}
                                        on_blur={on_field_blur(SessionDraftField::Mtu)}
                                        placeholder={i18n.t("stage2.field.mtu.placeholder")}
                                        disabled={loading}
                                    />
                                </ShellLine>
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
                                    "{}{}",
                                    if draft.mp_bgp { i18n.t("stage3.review.bgp.mpbgp") } else { i18n.t("stage3.review.bgp.separate") },
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
                            if let Some(session) = &auth_summary {
                                <>
                                    <span class="autopeer-status-pill">{format!("AS{}", session.asn)}</span>
                                    <span class="autopeer-node-badge">
                                        {format!("{} via {}", session.effective_mnt, session.auth_method.label)}
                                    </span>
                                </>
                            }
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
                                    {render_flow_steps(&i18n, active_stage)}
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
                                    if let Some(session) = &auth_summary {
                                        <p class="text-secondary">
                                            {i18n
                                                .t("sidebar.session_authed_template")
                                                .replace("{mnt}", &session.effective_mnt)
                                                .replace("{label}", &session.auth_method.label)}
                                        </p>
                                    }
                                </div>
                            </article>

                            if let Some(host_session) = &host_summary {
                                <article class="peering-card autopeer-panel autopeer-panel--compact">
                                    <div class="autopeer-panel-header">
                                        <p class="autopeer-panel-kicker">{i18n.t("sidebar.support_kicker")}</p>
                                        <h3 class="autopeer-panel-title">{format!("{}{}", i18n.t("sidebar.host_asn_prefix"), host_session.asn)}</h3>
                                        <p class="text-secondary">
                                            {i18n
                                                .t("sidebar.host_authed_template")
                                                .replace("{mnt}", &host_session.effective_mnt)
                                                .replace("{label}", &host_session.auth_method.label)}
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
                                                <p class="text-secondary">{message}</p>
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
                <h2 class="title title-flex">
                    <a href={(*autopeer_site_href).clone()} class="title-link">{i18n.t("app.title")}</a>
                    <span class="title-footnote">
                        {i18n.t("app.title.footnote")}
                        {" / "}
                        <a href={(*looking_glass_site_href).clone()} class="autopeer-title-nav">{i18n.t("nav.looking_glass")}</a>
                    </span>
                </h2>
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

    use common::auto_peer::{AuthMethod, AuthMethodKind, PeeringStrategy};

    use super::{
        Peer6AddressKind, SessionDraftLiveValidation, SessionDraftToggleGroup,
        autopeer_node_endpoint_port, detect_peer6_address_kind, displayed_peer_config_stage,
        retire_button_text, session_details_live_validation, toggle_group_key,
    };
    use crate::{
        controller::{configured_href, filter_supported_methods, validate_ssh_signature_input},
        store::{PeerConfigStage, SessionDraft},
    };

    #[test]
    fn prefers_runtime_configured_link_over_fallback() {
        assert_eq!(
            configured_href(Some("https://network.owo.li"), "https://lg.owo.li/"),
            "https://network.owo.li"
        );
    }

    #[test]
    fn hides_oidc_methods_when_runtime_config_disables_them() {
        let methods = vec![
            AuthMethod {
                kind: AuthMethodKind::RegistrySsh,
                label: "Registry SSH".into(),
                description: "SSH".into(),
                ..AuthMethod::default()
            },
            AuthMethod {
                kind: AuthMethodKind::Oidc,
                label: "Kioubit".into(),
                description: "OIDC".into(),
                provider: Some("kioubit".into()),
                ..AuthMethod::default()
            },
        ];

        let filtered = filter_supported_methods(methods, false);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].kind, AuthMethodKind::RegistrySsh);
    }

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
    fn live_validation_flags_ipv4_only_without_peer4_after_toggle_interaction() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched =
            BTreeSet::from([toggle_group_key(SessionDraftToggleGroup::Families).to_string()]);

        let validation = session_details_live_validation(&draft, &touched);

        assert_eq!(
            validation.bgp_message,
            Some("validation.peer4.required".into())
        );
        assert!(validation.highlight_peer4);
        assert!(validation.highlight_ipv4);
        assert!(!validation.highlight_mp_bgp);
    }

    #[test]
    fn live_validation_flags_mp_bgp_without_peer6_after_toggle_interaction() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "Cbefg96Owv1Xk/jrUExO3i5OeUSlsdirv4ONenEnNXc=".into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            peering_strategy: PeeringStrategy::FullTable,
            ..SessionDraft::default()
        };
        let touched = BTreeSet::from([toggle_group_key(SessionDraftToggleGroup::Bgp).to_string()]);

        let validation = session_details_live_validation(&draft, &touched);

        assert_eq!(
            validation.bgp_message,
            Some("validation.peer6.required_mp_bgp".into())
        );
        assert!(validation.highlight_peer6);
        assert!(validation.highlight_mp_bgp);
    }

    #[test]
    fn live_validation_stays_quiet_for_untouched_default_draft() {
        let validation =
            session_details_live_validation(&SessionDraft::default(), &BTreeSet::new());

        assert_eq!(validation, SessionDraftLiveValidation::default());
    }

    #[test]
    fn rejects_raw_challenge_text_in_ssh_signature_field() {
        assert_eq!(
            validate_ssh_signature_input(
                "dn42-autopeer challenge\nasn: 4242421024\nchallenge_id: example\nissued_at: 2026-04-18T12:42:04.075Z"
            ),
            Err("error.ssh.unsigned_challenge"),
        );
    }

    #[test]
    fn accepts_armored_ssh_signature_blocks() {
        assert_eq!(
            validate_ssh_signature_input(
                "-----BEGIN SSH SIGNATURE-----\nZm9v\n-----END SSH SIGNATURE-----"
            ),
            Ok(()),
        );
    }
}
