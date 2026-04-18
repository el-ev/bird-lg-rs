use std::collections::BTreeSet;

use common::{
    auto_peer::{
        ALL_PEERING_STRATEGIES, AuthMethodKind, NodeView, OperationStatus, PeeringStrategy,
        SessionState, SessionView,
    },
    models::PeeringInfo,
};
use ui_components::shell::{
    ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect, ShellToggle,
};
use web_sys::{HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::{
    config::{AUTOPEER_BASE_PATH, matches_autopeer_path},
    controller::{default_pgp_key, sync_create_draft, use_autopeer_controller},
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

fn render_command_block(label: &str, command: String) -> Html {
    render_readonly_block(label, command)
}

fn field_key(field: SessionDraftField) -> &'static str {
    match field {
        SessionDraftField::Endpoint => "endpoint",
        SessionDraftField::WgPublicKey => "wg_public_key",
        SessionDraftField::Port => "port",
        SessionDraftField::Peer4 => "peer4",
        SessionDraftField::Peer6 => "peer6",
        SessionDraftField::Own6 => "own6",
        SessionDraftField::Keepalive => "keepalive",
        SessionDraftField::Mtu => "mtu",
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

fn render_error(error: &Option<String>) -> Html {
    if let Some(error) = error {
        html! {
            <ShellLine>
                <span class="error-message">{error}</span>
            </ShellLine>
        }
    } else {
        Html::default()
    }
}

fn render_loading(loading: bool, loading_message: Option<&str>) -> Html {
    if loading {
        let message = loading_message.unwrap_or("Working...");
        html! {
            <ShellLine>
                <span class="text-secondary">{message}</span>
            </ShellLine>
        }
    } else {
        Html::default()
    }
}

fn autopeer_home_href_from_parts(protocol: &str, host: &str, pathname: &str) -> String {
    let path = if matches_autopeer_path(pathname) {
        AUTOPEER_BASE_PATH
    } else {
        "/"
    };
    format!("{protocol}//{host}{path}")
}

fn looking_glass_href_from_parts(protocol: &str, host: &str, pathname: &str) -> String {
    if let Some(rest) = host.strip_prefix("autopeer.") {
        format!("{protocol}//lg.{rest}/")
    } else if matches_autopeer_path(pathname) {
        format!("{protocol}//{host}/")
    } else {
        format!("{protocol}//{host}/")
    }
}

fn autopeer_home_href() -> String {
    web_sys::window()
        .and_then(|window| {
            let location = window.location();
            let protocol = location.protocol().ok()?;
            let host = location.host().ok()?;
            let pathname = location.pathname().ok()?;
            Some(autopeer_home_href_from_parts(&protocol, &host, &pathname))
        })
        .unwrap_or_else(|| "/".to_string())
}

fn looking_glass_href() -> String {
    web_sys::window()
        .and_then(|window| {
            let location = window.location();
            let protocol = location.protocol().ok()?;
            let host = location.host().ok()?;
            let pathname = location.pathname().ok()?;
            Some(looking_glass_href_from_parts(&protocol, &host, &pathname))
        })
        .unwrap_or_else(|| "/".to_string())
}

fn session_for_node<'a>(node_name: &str, sessions: &'a [SessionView]) -> Option<&'a SessionView> {
    sessions.iter().find(|session| session.node == node_name)
}

fn humanize_token(token: &str) -> String {
    match token {
        "N" => "North".to_string(),
        "S" => "South".to_string(),
        "E" => "East".to_string(),
        "W" => "West".to_string(),
        "NE" => "Northeast".to_string(),
        "NW" => "Northwest".to_string(),
        "SE" => "Southeast".to_string(),
        "SW" => "Southwest".to_string(),
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

fn humanize_region(region: &Option<String>) -> Option<String> {
    region.as_ref().map(|value| {
        value
            .split('_')
            .map(humanize_token)
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn humanize_ip_support(value: &str) -> &'static str {
    match value {
        "ipv4" => "IPv4 transport",
        "ipv6" => "IPv6 transport",
        _ => "Dual-stack transport",
    }
}

fn node_context_line(node: &NodeView) -> String {
    let mut parts = Vec::new();

    if let Some(region) = humanize_region(&node.region) {
        parts.push(region);
    }
    if let Some(country) = &node.country {
        parts.push(country.to_string());
    }

    if parts.is_empty() {
        humanize_ip_support(&node.ip_support).to_string()
    } else {
        parts.join(", ")
    }
}

fn node_review_line(node: &NodeView) -> String {
    let context = node_context_line(node);
    if context.is_empty() {
        node.name.clone()
    } else {
        format!("{} ({context})", node.name)
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

fn render_inventory_peering_review(node: Option<&NodeView>, active_asn: &str) -> Html {
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
            <p class="autopeer-review-section-title">{"Our node details"}</p>
            <dl class="peering-grid autopeer-review-peering-grid">
                {render_peering_field("Our endpoint", node_endpoint.as_deref())}
                {render_peering_field("Our IPv4", peering.ipv4.as_deref())}
                {render_peering_field("Our IPv6", peering.ipv6.as_deref())}
                {render_peering_field("Our link-local IPv6", peering.link_local_ipv6.as_deref())}
                {render_peering_field("Our WireGuard public key", peering.wg_pubkey.as_deref())}
                {render_peering_field("Our node note", peering.comment.as_deref())}
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
        common::auto_peer::OperationState::PendingPullRequest => 0,
        common::auto_peer::OperationState::PendingChecks => 1,
        common::auto_peer::OperationState::PendingMerge => 2,
        common::auto_peer::OperationState::Merged | common::auto_peer::OperationState::Applying => {
            3
        }
        common::auto_peer::OperationState::Completed => 4,
        common::auto_peer::OperationState::Failed | common::auto_peer::OperationState::Conflict => {
            2
        }
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

fn retire_button_text(retire_confirmation_armed: bool) -> &'static str {
    if retire_confirmation_armed {
        "Confirm Retirement"
    } else {
        "Retire This Session"
    }
}

fn render_flow_steps(stage: PeerConfigStage) -> Html {
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
                            <strong>{candidate.title()}</strong>
                            <span>{candidate.description()}</span>
                        </span>
                    </li>
                }
            })}
        </ol>
    }
}

fn render_operation_progress(operation: &OperationStatus) -> Html {
    let labels = ["Branch", "Checks", "Merge", "Apply", "Done"];
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
                        <span>{label.to_string()}</span>
                    </li>
                }
            })}
        </ol>
    }
}

#[function_component(AutoPeerPage)]
pub fn auto_peer_page() -> Html {
    let default_autopeer_home_href = autopeer_home_href();
    let default_looking_glass_href = looking_glass_href();
    let controller =
        use_autopeer_controller(default_autopeer_home_href, default_looking_glass_href);
    let autopeer_site_href = controller.autopeer_site_href.clone();
    let looking_glass_site_href = controller.looking_glass_site_href.clone();
    let oidc_methods = controller.oidc_methods.clone();
    let step = controller.step.clone();
    let asn = controller.asn.clone();
    let challenge_text = controller.challenge_text.clone();
    let methods = controller.methods.clone();
    let selected_method = controller.selected_method.clone();
    let auth_session = controller.auth_session.clone();
    let host_session = controller.host_session.clone();
    let nodes = controller.nodes.clone();
    let sessions = controller.sessions.clone();
    let draft = controller.draft.clone();
    let touched_fields = controller.touched_fields.clone();
    let editing_node = controller.editing_node.clone();
    let config_stage = controller.config_stage.clone();
    let retire_confirmation_armed = controller.retire_confirmation_armed.clone();
    let operation = controller.operation.clone();
    let error = controller.error.clone();
    let loading = controller.loading.clone();
    let loading_message = controller.loading_message.clone();
    let impersonate_asn = controller.impersonate_asn.clone();
    let impersonate_mnt = controller.impersonate_mnt.clone();
    let ssh_signature = controller.ssh_signature.clone();
    let selected_pgp_key = controller.selected_pgp_key.clone();
    let pgp_public_key = controller.pgp_public_key.clone();
    let pgp_signed_message = controller.pgp_signed_message.clone();
    let on_asn_change = controller.on_asn_change.clone();
    let on_submit_asn = controller.on_submit_asn.clone();
    let on_asn_keydown = controller.on_asn_keydown.clone();
    let on_enter_oidc = controller.on_enter_oidc.clone();
    let on_select_method = controller.on_select_method.clone();
    let on_select_method_back = controller.on_select_method_back.clone();
    let on_verify_back = controller.on_verify_back.clone();
    let on_verify = controller.on_verify.clone();
    let on_refresh = controller.on_refresh.clone();
    let on_logout = controller.on_logout.clone();
    let on_impersonate_asn_change = controller.on_impersonate_asn_change.clone();
    let on_impersonate_mnt_change = controller.on_impersonate_mnt_change.clone();
    let on_impersonate = controller.on_impersonate.clone();
    let on_return_to_host = controller.on_return_to_host.clone();
    let on_submit_session = controller.on_submit_session.clone();
    let on_retire_selected_session = controller.on_retire_selected_session.clone();

    let content = match &*step {
        AutoPeerStep::LoadingConfig => html! {
            <div class="autopeer-step">
                <ShellLine>
                    <ShellPrompt>{"autopeer"}</ShellPrompt>
                    {" Loading runtime configuration"}
                </ShellLine>
                {render_loading(true, Some("Loading runtime configuration..."))}
                {render_error(&error)}
            </div>
        },
        AutoPeerStep::EnterAsn => html! {
            <div class="autopeer-step">
                <ShellLine>
                    <ShellPrompt>{"autopeer"}</ShellPrompt>
                    {" Enter your DN42 ASN for registry SSH or PGP auth"}
                </ShellLine>
                <ShellLine>
                    <ShellPrompt>{"asn"}</ShellPrompt>
                    {" "}
                    <ShellInput
                        value={(*asn).clone()}
                        on_change={on_asn_change}
                        placeholder="424242xxxx"
                        disabled={*loading}
                        on_keydown={on_asn_keydown}
                    />
                </ShellLine>
                {render_loading(*loading, loading_message.as_deref())}
                {render_error(&error)}
                <ShellLine>
                    <ShellButton
                        text="Find Registry Auth Methods"
                        onclick={on_submit_asn}
                        disabled={*loading || asn.trim().is_empty()}
                    />
                </ShellLine>
                if !oidc_methods.is_empty() {
                    <div class="autopeer-entry-alt">
                        <div class="autopeer-entry-alt-copy">
                            {"Or sign in with your identity provider and let us derive your ASN automatically."}
                        </div>
                        <div class="autopeer-challenge-list">
                            {for oidc_methods.iter().map(|method| {
                                let on_enter_oidc = on_enter_oidc.clone();
                                let loading_for_button = loading.clone();
                                let method = method.clone();
                                let method_copy = method.clone();
                                let onclick = Callback::from(move |_| {
                                    on_enter_oidc.emit(method_copy.clone());
                                });

                                html! {
                                    <ShellLine>
                                        <ShellButton
                                            text={format!("Continue with {}", method.label)}
                                            onclick={onclick}
                                            disabled={*loading_for_button}
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
                        <ShellPrompt>{"autopeer"}</ShellPrompt>
                        {format!(" We found registry auth methods for AS{}", *asn)}
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
                                        disabled={*loading}
                                    />
                                    <span class="autopeer-method-desc">
                                        {format!(" - {}", method.description)}
                                    </span>
                                </ShellLine>
                            }
                        })}
                    </div>
                    {render_loading(*loading, loading_message.as_deref())}
                    {render_error(&error)}
                    <ShellLine>
                        <ShellButton
                            text="Back"
                            onclick={on_select_method_back.clone()}
                            disabled={*loading}
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
                                            {"We could not find any registry SSH key fingerprints for your ASN."}
                                        </span>
                                    </ShellLine>
                                } else if method.ssh_fingerprints.len() == 1 {
                                    <ShellLine>
                                        <ShellPrompt>{"key"}</ShellPrompt>
                                        {format!(" Match your SSH key {}", method.ssh_fingerprints[0])}
                                    </ShellLine>
                                } else {
                                    <ShellLine>
                                        <ShellPrompt>{"keys"}</ShellPrompt>
                                        {format!(
                                            " Match one of your SSH keys: {}",
                                            method.ssh_fingerprints.join(", ")
                                        )}
                                    </ShellLine>
                                }
                                if let Some(challenge) = &*challenge_text {
                                    {render_command_block(
                                        "Create your SSH signature",
                                        ssh_sign_command(challenge),
                                    )}
                                }
                                <ShellLine>
                                    <ShellPrompt>{"signature"}</ShellPrompt>
                                    {" Run the command above, then paste your detached SSH signature"}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*ssh_signature).clone()}
                                        on_change={on_change}
                                        placeholder="-----BEGIN SSH SIGNATURE-----"
                                        disabled={*loading}
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
                                            {"We could not find any registry PGP fingerprints for your ASN."}
                                        </span>
                                    </ShellLine>
                                } else if method.pgp_fingerprints.len() == 1 {
                                    <ShellLine>
                                        <ShellPrompt>{"key"}</ShellPrompt>
                                        {format!(" Use your key {}", method.pgp_fingerprints[0])}
                                    </ShellLine>
                                } else {
                                    <ShellLine>
                                        <ShellPrompt>{"key"}</ShellPrompt>
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
                                                {"Clear-sign the exact challenge text with your matching key, then export that same public key and paste both outputs below."}
                                            </span>
                                        </ShellLine>
                                        {render_readonly_block(
                                            "Exact challenge text",
                                            challenge.clone(),
                                        )}
                                        {render_command_block(
                                            "Clear-sign your challenge",
                                            pgp_sign_command(challenge, &selected_key_value),
                                        )}
                                    </>
                                } else {
                                    <ShellLine>
                                        <span class="text-secondary">
                                            {"Clear-sign the exact challenge text with your matching key, then export that same public key and paste both outputs below."}
                                        </span>
                                    </ShellLine>
                                }
                                <ShellLine>
                                    <ShellPrompt>{"signed"}</ShellPrompt>
                                    {" Paste your full clear-signed challenge from the command above"}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*pgp_signed_message).clone()}
                                        on_change={on_signed_change}
                                        placeholder="-----BEGIN PGP SIGNED MESSAGE-----"
                                        disabled={*loading}
                                        multiline=true
                                        rows={12}
                                    />
                                </ShellLine>
                                {render_command_block(
                                    "Export your public key",
                                    pgp_export_command(&selected_key_value),
                                )}
                                <ShellLine>
                                    <ShellPrompt>{"pubkey"}</ShellPrompt>
                                    {" Paste your ASCII-armored public key from the export command above"}
                                </ShellLine>
                                <ShellLine>
                                    <ShellInput
                                        value={(*pgp_public_key).clone()}
                                        on_change={on_pubkey_change}
                                        placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----"
                                        disabled={*loading}
                                        multiline=true
                                        rows={8}
                                    />
                                </ShellLine>
                            </>
                        }
                    }
                    AuthMethodKind::Oidc => {
                        html! {
                            <>
                                <ShellLine>
                                    <ShellPrompt>{"login"}</ShellPrompt>
                                    {format!(" Continue to {} in your browser", method.label)}
                                </ShellLine>
                                <ShellLine>
                                    <span class="text-secondary">
                                        {"We will redirect you to your provider, then bring you back here after it proves your ASN and maintainer claims."}
                                    </span>
                                </ShellLine>
                            </>
                        }
                    }
                    AuthMethodKind::HostImpersonation => html! {
                        <ShellLine>
                            <span class="text-secondary">
                                {"Impersonation is available after you authenticate one of our configured host ASNs."}
                            </span>
                        </ShellLine>
                    },
                };
                let verify_button_text = if method.kind == AuthMethodKind::Oidc {
                    format!("Continue to {}", method.label)
                } else {
                    "Verify".to_string()
                };

                html! {
                    <div class="autopeer-step">
                        <ShellLine>
                            <ShellPrompt>{"auth"}</ShellPrompt>
                            {format!(" {} for AS{}", method.label, *asn)}
                        </ShellLine>
                        {verification_fields}
                        {render_loading(*loading, loading_message.as_deref())}
                        {render_error(&error)}
                        <ShellLine>
                            <ShellButton
                                text="Back"
                                onclick={on_verify_back.clone()}
                                disabled={*loading}
                            />
                            {" "}
                            <ShellButton text={verify_button_text} onclick={on_verify} disabled={*loading} />
                        </ShellLine>
                    </div>
                }
            } else {
                html! {
                    <div class="autopeer-step">
                        <ShellLine>
                            <span class="error-message">{"Choose an authentication method first."}</span>
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
            let retire_confirmation_armed_value = *retire_confirmation_armed;
            let active_asn = auth_summary
                .as_ref()
                .map(|session| session.asn.clone())
                .unwrap_or_else(|| (*asn).clone());
            let draft_is_valid = draft.to_spec().is_ok();
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
                        "Only needed when your peer IPv6 address is link-local".to_string()
                    })
                }
                _ => "Only needed when your peer IPv6 address is link-local".to_string(),
            };

            let on_cancel_edit = {
                let editing_node = editing_node.clone();
                let draft = draft.clone();
                let sessions = sessions.clone();
                let nodes = nodes.clone();
                let auth_session = auth_session.clone();
                let asn = asn.clone();
                let config_stage = config_stage.clone();
                let touched_fields = touched_fields.clone();
                Callback::from(move |_| {
                    editing_node.set(None);
                    config_stage.set(PeerConfigStage::SelectNode);
                    touched_fields.set(BTreeSet::new());
                    let active_asn = auth_session
                        .as_ref()
                        .as_ref()
                        .map(|session| session.asn.as_str())
                        .unwrap_or_else(|| asn.as_str());
                    draft.set(sync_create_draft(active_asn, &nodes, &sessions, &draft));
                })
            };

            let update_text_field = |setter: fn(&mut SessionDraft) -> &mut String| {
                let draft = draft.clone();
                Callback::from(move |value: String| {
                    let mut next = (*draft).clone();
                    *setter(&mut next) = value;
                    draft.set(next);
                })
            };

            let on_field_blur = |field: SessionDraftField| {
                let touched_fields = touched_fields.clone();
                Callback::from(move |_: FocusEvent| {
                    let mut next = (*touched_fields).clone();
                    next.insert(field_key(field).to_string());
                    touched_fields.set(next);
                })
            };

            let input_class = |field: SessionDraftField| {
                let key = field_key(field);
                if touched_fields.contains(key) && draft.field_error(field).is_some() {
                    classes!("shell-input--invalid")
                } else {
                    Classes::new()
                }
            };

            let input_frame_class = |field: SessionDraftField| {
                let key = field_key(field);
                if touched_fields.contains(key) && draft.field_error(field).is_some() {
                    classes!("shell-input-frame--invalid")
                } else {
                    Classes::new()
                }
            };

            let on_peer6_change = {
                let draft = draft.clone();
                Callback::from(move |value: String| {
                    let mut next = (*draft).clone();
                    next.peer6 = value;
                    if !next.peer6_is_link_local() {
                        next.own6.clear();
                    }
                    draft.set(next);
                })
            };

            let on_toggle_ipv4 = {
                let draft = draft.clone();
                Callback::from(move |_| {
                    let mut next = (*draft).clone();
                    next.ipv4 = !next.ipv4;
                    draft.set(next);
                })
            };

            let on_toggle_ipv6 = {
                let draft = draft.clone();
                Callback::from(move |_| {
                    let mut next = (*draft).clone();
                    next.ipv6 = !next.ipv6;
                    draft.set(next);
                })
            };

            let on_toggle_mp_bgp = {
                let draft = draft.clone();
                Callback::from(move |_: ()| {
                    let mut next = (*draft).clone();
                    next.mp_bgp = !next.mp_bgp;
                    if !next.mp_bgp {
                        next.extended_next_hop = false;
                    }
                    draft.set(next);
                })
            };

            let on_toggle_extended_next_hop = {
                let draft = draft.clone();
                Callback::from(move |_| {
                    let mut next = (*draft).clone();
                    next.extended_next_hop = !next.extended_next_hop;
                    if next.extended_next_hop {
                        next.mp_bgp = true;
                    }
                    draft.set(next);
                })
            };

            let on_change_peering_strategy = {
                let draft = draft.clone();
                Callback::from(move |event: Event| {
                    let select: HtmlSelectElement = event.target_unchecked_into();
                    let mut next = (*draft).clone();
                    next.peering_strategy = PeeringStrategy::from_value(&select.value())
                        .unwrap_or(PeeringStrategy::FullTable);
                    draft.set(next);
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
                Callback::from(move |_| {
                    if editing_node.is_none() && draft.node.trim().is_empty() {
                        error.set(Some(
                            "Choose one of our nodes before you continue".to_string(),
                        ));
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
                            <p class="autopeer-panel-kicker">{"Stage 1"}</p>
                            <h3 class="autopeer-panel-title">{"Choose one of our nodes"}</h3>
                            <p class="text-secondary">
                                {"Choose a node in our network. Empty nodes let you create a session. Existing sessions open in-place so you can update them, and manual sessions are adopted into autopeer automatically when you save. In-flight nodes stay read-only."}
                            </p>
                        </div>
                        if nodes.is_empty() {
                            <div class="autopeer-empty-state">
                                <p>{"We did not find any autopeer-enabled nodes for your ASN."}</p>
                                <p class="text-secondary">
                                    {"Refresh our inventory or check our autopeer policy if that looks wrong."}
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
                                    let selectable = matches!(
                                        node_session.as_ref().map(|session| &session.state),
                                        None | Some(SessionState::Managed) | Some(SessionState::Manual)
                                    );
                                    let state_label = node_session
                                        .as_ref()
                                        .map(|session| session.state.label())
                                        .unwrap_or("Available");
                                    let state_note = match node_session.as_ref().map(|session| &session.state) {
                                        None => "Create your session on this node.",
                                        Some(SessionState::Managed) => "Open this node to update or retire your managed session.",
                                        Some(SessionState::Manual) => "Open this node to review the current repo config. Saving it will adopt the session into autopeer automatically.",
                                        Some(SessionState::PendingPr) => "A change for your session is already in progress here.",
                                        Some(SessionState::Conflict) => "Our repo is in conflict for this node.",
                                    };
                                    let onclick = Callback::from(move |_| {
                                        error.set(None);
                                        match node_session_for_click.as_ref().map(|session| &session.state) {
                                            None => {
                                                editing_node.set(None);
                                                let mut next = (*draft).clone();
                                                next.node = node_value.name.clone();
                                                draft.set(next);
                                                config_stage.set(PeerConfigStage::SessionDetails);
                                            }
                                            Some(SessionState::Managed) | Some(SessionState::Manual) => {
                                                let Some(spec) = node_session_for_click.as_ref().and_then(|session| session.spec.clone()) else {
                                                    error.set(Some("Your current session is missing config details".to_string()));
                                                    return;
                                                };
                                                editing_node.set(Some(node_value.name.clone()));
                                                draft.set(SessionDraft::from_session(&node_value.name, &spec));
                                                config_stage.set(PeerConfigStage::SessionDetails);
                                            }
                                            Some(SessionState::PendingPr) => {
                                                error.set(Some("Wait for the in-flight change on this node to finish first".to_string()));
                                            }
                                            Some(SessionState::Conflict) => {
                                                error.set(Some("This node is blocked by a conflict in our repo".to_string()));
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
                                            disabled={*loading || !selectable}
                                        >
                                            <span class="autopeer-node-option-head">
                                                <strong class="autopeer-node-name">{node.name.clone()}</strong>
                                                <span class="autopeer-node-option-status">
                                                    <span class="autopeer-node-badge">{node.ip_support.clone()}</span>
                                                    <span class="autopeer-status-pill">{state_label}</span>
                                                </span>
                                            </span>
                                            <span class="autopeer-node-meta">
                                                {node_context_line(node)}
                                            </span>
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
                        {render_error(&error)}
                    </article>
                },
                PeerConfigStage::SessionDetails => html! {
                    <article class="peering-card autopeer-panel">
                        <div class="autopeer-panel-header">
                            <p class="autopeer-panel-kicker">{"Stage 2"}</p>
                            <h3 class="autopeer-panel-title">
                                {
                                    if let Some(node) = &editing_node_value {
                                        format!("Update or retire your session on {}", node)
                                    } else if let Some(node) = &selected_node {
                                        format!("Set up your session on {}", node.name)
                                    } else {
                                        "Set up your new session".to_string()
                                    }
                                }
                            </h3>
                            if editing_node_value.is_some() {
                                <p class="text-secondary">
                                    {"You already have a managed session on this node. Update your peering details below, or retire the session if you no longer want it here."}
                                </p>
                            }
                        </div>

                        if let Some(node) = &selected_node {
                            <div class="autopeer-node-summary">
                                <div>
                                    <strong>{node_context_line(node)}</strong>
                                    if let Some(comment) = &node.comment {
                                        <p class="text-secondary">{comment.clone()}</p>
                                    }
                                </div>
                                if editing_node_value.is_none() {
                                    <ShellButton text="Choose Another Node" onclick={on_change_node.clone()} disabled={*loading} />
                                }
                            </div>
                        }

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{"Connection"}</span>
                            <ShellLine>
                                <ShellPrompt>{"Your endpoint"}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.endpoint.clone()}
                                    on_change={update_text_field(|draft| &mut draft.endpoint)}
                                    class={input_class(SessionDraftField::Endpoint)}
                                    frame_class={input_frame_class(SessionDraftField::Endpoint)}
                                    on_blur={on_field_blur(SessionDraftField::Endpoint)}
                                    placeholder="Hostname or IP:port of your router"
                                    disabled={*loading}
                                />
                            </ShellLine>
                            <ShellLine>
                                <ShellPrompt>{"Your WireGuard key"}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.wg_public_key.clone()}
                                    on_change={update_text_field(|draft| &mut draft.wg_public_key)}
                                    class={input_class(SessionDraftField::WgPublicKey)}
                                    frame_class={input_frame_class(SessionDraftField::WgPublicKey)}
                                    on_blur={on_field_blur(SessionDraftField::WgPublicKey)}
                                    placeholder="Base64 public key from your router"
                                    disabled={*loading}
                                />
                            </ShellLine>
                            <ShellLine>
                                <ShellPrompt>{"WireGuard port"}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.port.clone()}
                                    on_change={update_text_field(|draft| &mut draft.port)}
                                    class={input_class(SessionDraftField::Port)}
                                    frame_class={input_frame_class(SessionDraftField::Port)}
                                    on_blur={on_field_blur(SessionDraftField::Port)}
                                    placeholder={draft.resolved_port(&active_asn)}
                                    disabled={*loading}
                                />
                            </ShellLine>
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{"Tunnel Addresses"}</span>
                            <p class="text-secondary">
                                {"Use the addresses you configured on your side. IPv6 can be either ULA like `fd55:...` or link-local like `fe80:...`."}
                            </p>
                            <ShellLine>
                                <ShellPrompt>{"Peer IPv4 address"}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.peer4.clone()}
                                    on_change={update_text_field(|draft| &mut draft.peer4)}
                                    class={input_class(SessionDraftField::Peer4)}
                                    frame_class={input_frame_class(SessionDraftField::Peer4)}
                                    on_blur={on_field_blur(SessionDraftField::Peer4)}
                                    placeholder="Optional DN42 IPv4 address on your side"
                                    disabled={*loading}
                                />
                            </ShellLine>
                            <ShellLine>
                                <ShellPrompt>{"Peer IPv6 address"}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.peer6.clone()}
                                    on_change={on_peer6_change}
                                    class={input_class(SessionDraftField::Peer6)}
                                    frame_class={input_frame_class(SessionDraftField::Peer6)}
                                    on_blur={on_field_blur(SessionDraftField::Peer6)}
                                    placeholder="ULA or link-local, e.g. fd55:dead:beef::3 or fe80::1234"
                                    disabled={*loading}
                                />
                            </ShellLine>
                            if peer6_kind == Some(Peer6AddressKind::LinkLocal) {
                                <ShellLine>
                                    <ShellPrompt>{"Our link-local IPv6"}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.own6.clone()}
                                        on_change={update_text_field(|draft| &mut draft.own6)}
                                        class={input_class(SessionDraftField::Own6)}
                                        frame_class={input_frame_class(SessionDraftField::Own6)}
                                        on_blur={on_field_blur(SessionDraftField::Own6)}
                                        placeholder={own6_placeholder}
                                        disabled={*loading}
                                    />
                                </ShellLine>
                            } else if peer6_kind == Some(Peer6AddressKind::Ula) {
                                <ShellLine>
                                    <ShellPrompt>{"Our node IPv6"}</ShellPrompt>
                                    {" "}
                                    <span class="text-secondary">
                                        {node_inventory_ipv6.clone().unwrap_or_else(|| "Our inventory does not expose an IPv6 address for this node".to_string())}
                                    </span>
                                </ShellLine>
                            }
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{"Route Families"}</span>
                            <p class="text-secondary">
                                {"Choose which DN42 route families your session should carry."}
                            </p>
                            <ShellLine>
                                <ShellPrompt>{"Families"}</ShellPrompt>
                                {" "}
                                <span class="autopeer-toggle-row">
                                    <ShellToggle
                                        active={draft.ipv4}
                                        on_toggle={on_toggle_ipv4}
                                        label="IPv4 routes"
                                    />
                                    {" "}
                                    <ShellToggle
                                        active={draft.ipv6}
                                        on_toggle={on_toggle_ipv6}
                                        label="IPv6 routes"
                                    />
                                </span>
                            </ShellLine>
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{"BGP Behavior"}</span>
                            <p class="text-secondary">
                                {"MP-BGP uses your IPv6 address for a combined IPv4+IPv6 session. If you disable it, we will generate separate BGP sessions. Extended Next Hop only applies to MP-BGP."}
                            </p>
                            <ShellLine>
                                <ShellPrompt>{"Features"}</ShellPrompt>
                                {" "}
                                <span class="autopeer-toggle-row">
                                    <ShellToggle
                                        active={draft.mp_bgp}
                                        on_toggle={on_toggle_mp_bgp}
                                        label="MP-BGP"
                                    />
                                    {" "}
                                    <ShellToggle
                                        active={draft.extended_next_hop}
                                        on_toggle={on_toggle_extended_next_hop}
                                        label="Extended Next Hop"
                                    />
                                </span>
                            </ShellLine>
                        </div>

                        <div class="autopeer-form-section">
                            <span class="autopeer-section-label">{"Routing Policy"}</span>
                            <p class="text-secondary">
                                {draft.peering_strategy.description()}
                            </p>
                            <ShellLine>
                                <ShellPrompt>{"Policy"}</ShellPrompt>
                                {" "}
                                <ShellSelect
                                    value={draft.peering_strategy.as_str()}
                                    on_change={on_change_peering_strategy}
                                >
                                    {
                                        for ALL_PEERING_STRATEGIES.iter().map(|strategy| html! {
                                            <option value={strategy.as_str()}>{strategy.label()}</option>
                                        })
                                    }
                                </ShellSelect>
                            </ShellLine>
                        </div>

                        <details class="autopeer-advanced">
                            <summary>{"Advanced options"}</summary>
                            <div class="autopeer-form-section autopeer-form-section--advanced">
                                <ShellLine>
                                    <ShellPrompt>{"comment"}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.comment.clone()}
                                        on_change={update_text_field(|draft| &mut draft.comment)}
                                        placeholder="Optional note about your session"
                                        disabled={*loading}
                                    />
                                </ShellLine>
                                <ShellLine>
                                    <ShellPrompt>{"Persistent keepalive"}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.keepalive.clone()}
                                        on_change={update_text_field(|draft| &mut draft.keepalive)}
                                        class={input_class(SessionDraftField::Keepalive)}
                                        frame_class={input_frame_class(SessionDraftField::Keepalive)}
                                        on_blur={on_field_blur(SessionDraftField::Keepalive)}
                                        placeholder="Optional keepalive in seconds for your router"
                                        disabled={*loading}
                                    />
                                </ShellLine>
                                <ShellLine>
                                    <ShellPrompt>{"Interface MTU"}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.mtu.clone()}
                                        on_change={update_text_field(|draft| &mut draft.mtu)}
                                        class={input_class(SessionDraftField::Mtu)}
                                        frame_class={input_frame_class(SessionDraftField::Mtu)}
                                        on_blur={on_field_blur(SessionDraftField::Mtu)}
                                        placeholder="Optional MTU"
                                        disabled={*loading}
                                    />
                                </ShellLine>
                            </div>
                        </details>

                        {render_loading(*loading, loading_message.as_deref())}
                        {render_error(&error)}

                        <div class="autopeer-inline-actions">
                            if editing_node_value.is_some() {
                                <ShellButton text="Cancel Edit" onclick={on_cancel_edit.clone()} disabled={*loading} />
                                <ShellButton
                                    text={retire_button_text(retire_confirmation_armed_value)}
                                    onclick={on_retire_selected_session.clone()}
                                    disabled={*loading}
                                />
                            } else {
                                <ShellButton text="Back To Nodes" onclick={on_change_node.clone()} disabled={*loading} />
                            }
                            <ShellButton
                                text={if editing_node_value.is_some() { "Review Your Update" } else { "Review Your Change" }}
                                onclick={on_continue_to_review}
                                disabled={
                                    *loading
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
                            <p class="autopeer-panel-kicker">{"Stage 3"}</p>
                            <h3 class="autopeer-panel-title">{"Review your change before we open the PR"}</h3>
                        </div>

                        <div class="autopeer-review-grid">
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"Our node"}</span>
                                <strong class="autopeer-review-value">
                                    {selected_node.as_ref().map(node_review_line).unwrap_or_else(|| "Not selected".to_string())}
                                </strong>
                            </div>
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"Your endpoint"}</span>
                                <strong class="autopeer-review-value">{draft.endpoint.clone()}</strong>
                            </div>
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"Your WireGuard key"}</span>
                                <strong class="autopeer-review-value">{draft.wg_public_key.clone()}</strong>
                            </div>
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"WireGuard port"}</span>
                                <strong class="autopeer-review-value">{draft.resolved_port(&active_asn)}</strong>
                            </div>
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"Route families"}</span>
                                <strong class="autopeer-review-value">{draft.families_label()}</strong>
                            </div>
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"BGP behavior"}</span>
                                <strong class="autopeer-review-value">
                                    {format!(
                                        "{}{}",
                                        if draft.mp_bgp { "MP-BGP" } else { "Separate IPv4/IPv6 sessions" },
                                        if draft.extended_next_hop { " + Extended Next Hop" } else { "" },
                                    )}
                                </strong>
                            </div>
                            <div class="autopeer-review-item">
                                <span class="autopeer-review-label">{"Routing policy"}</span>
                                <strong class="autopeer-review-value">{draft.peering_strategy.label()}</strong>
                            </div>
                            if !draft.peer4.trim().is_empty() {
                                <div class="autopeer-review-item">
                                    <span class="autopeer-review-label">{"Peer IPv4 address"}</span>
                                    <strong class="autopeer-review-value">{draft.peer4.clone()}</strong>
                                </div>
                            }
                            if !draft.peer6.trim().is_empty() {
                                <div class="autopeer-review-item">
                                    <span class="autopeer-review-label">{"Peer IPv6 address"}</span>
                                    <strong class="autopeer-review-value">{draft.peer6.clone()}</strong>
                                </div>
                            }
                            if !draft.own6.trim().is_empty() {
                                <div class="autopeer-review-item">
                                    <span class="autopeer-review-label">{"Our link-local IPv6"}</span>
                                    <strong class="autopeer-review-value">{draft.own6.clone()}</strong>
                                </div>
                            }
                            if !draft.keepalive.trim().is_empty() {
                                <div class="autopeer-review-item">
                                    <span class="autopeer-review-label">{"Persistent keepalive"}</span>
                                    <strong class="autopeer-review-value">{draft.keepalive.clone()}</strong>
                                </div>
                            }
                            if !draft.mtu.trim().is_empty() {
                                <div class="autopeer-review-item">
                                    <span class="autopeer-review-label">{"MTU"}</span>
                                    <strong class="autopeer-review-value">{draft.mtu.clone()}</strong>
                                </div>
                            }
                            if !draft.comment.trim().is_empty() {
                                <div class="autopeer-review-item">
                                    <span class="autopeer-review-label">{"Your note"}</span>
                                    <strong class="autopeer-review-value">{draft.comment.clone()}</strong>
                                </div>
                            }
                        </div>

                        {render_inventory_peering_review(selected_node.as_ref(), &active_asn)}

                        {render_loading(*loading, loading_message.as_deref())}
                        {render_error(&error)}

                        <div class="autopeer-inline-actions">
                            <ShellButton text="Back To Details" onclick={on_back_to_details} disabled={*loading} />
                            if editing_node_value.is_some() {
                                <ShellButton text="Cancel Edit" onclick={on_cancel_edit} disabled={*loading} />
                            } else {
                                <ShellButton text="Choose Another Node" onclick={on_change_node} disabled={*loading} />
                            }
                            <ShellButton
                                text={if editing_node_value.is_some() { "Open Update PR" } else { "Open Create PR" }}
                                onclick={on_submit_session}
                                disabled={
                                    *loading
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
                            <p class="autopeer-panel-kicker">{"Your Peering Flow"}</p>
                            <h3 class="autopeer-panel-title">
                                {if host_session_active {
                                    "Our host ASN stays read-only here"
                                } else if editing_node_value.is_some() {
                                    "Update your managed session"
                                } else {
                                    "Create or manage your sessions"
                                }}
                            </h3>
                            <p class="text-secondary">
                                {if host_session_active {
                                    "Our host ASN is only for support impersonation. Impersonate the ASN you want to manage before you create, update, or retire sessions."
                                } else {
                                    "Authenticate once, choose one of our nodes, then create a new session there or open your managed session to update or retire it."
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
                            <ShellButton text="Refresh" onclick={on_refresh.clone()} disabled={*loading} />
                            <ShellButton text="Logout" onclick={on_logout} disabled={*loading} />
                        </div>
                    </section>

                    <div class="autopeer-workspace">
                        <div class="autopeer-main">
                            if host_session_active {
                                <article class="peering-card autopeer-panel">
                                    <div class="autopeer-panel-header">
                                        <p class="autopeer-panel-kicker">{"Support Mode"}</p>
                                        <h3 class="autopeer-panel-title">{"Impersonate another ASN"}</h3>
                                        <p class="text-secondary">
                                            {"This host ASN only lets you support other networks. Use the controls on the right to impersonate the ASN you want to manage first."}
                                        </p>
                                    </div>
                                </article>
                            } else {
                                <>
                                    {render_flow_steps(active_stage)}
                                    {main_panel}
                                </>
                            }
                        </div>

                        <aside class="autopeer-sidebar">
                            <article class="peering-card autopeer-panel autopeer-panel--compact">
                                <div class="autopeer-panel-header">
                                    <p class="autopeer-panel-kicker">{"Your Session"}</p>
                                    <h3 class="autopeer-panel-title">
                                        {auth_summary.as_ref().map(|session| format!("AS{}", session.asn)).unwrap_or_else(|| "No active session".to_string())}
                                    </h3>
                                    if let Some(session) = &auth_summary {
                                        <p class="text-secondary">
                                            {format!(
                                                "You authenticated as {} via {}.",
                                                session.effective_mnt,
                                                session.auth_method.label
                                            )}
                                        </p>
                                    }
                                </div>
                            </article>

                            if let Some(host_session) = &host_summary {
                                <article class="peering-card autopeer-panel autopeer-panel--compact">
                                    <div class="autopeer-panel-header">
                                        <p class="autopeer-panel-kicker">{"Support Mode"}</p>
                                        <h3 class="autopeer-panel-title">{format!("Host ASN AS{}", host_session.asn)}</h3>
                                        <p class="text-secondary">
                                            {format!(
                                                "You authenticated as {} via {}. Use this only when you need to open or repair sessions for another ASN.",
                                                host_session.effective_mnt,
                                                host_session.auth_method.label
                                            )}
                                        </p>
                                    </div>
                                    <div class="autopeer-form-section autopeer-form-section--compact">
                                        <ShellLine>
                                            <ShellPrompt>{"impersonate_asn"}</ShellPrompt>
                                            {" "}
                                            <ShellInput
                                                value={(*impersonate_asn).clone()}
                                                on_change={on_impersonate_asn_change}
                                                placeholder="424242xxxx"
                                                disabled={*loading}
                                            />
                                        </ShellLine>
                                        <ShellLine>
                                            <ShellPrompt>{"effective_mnt"}</ShellPrompt>
                                            {" "}
                                            <ShellInput
                                                value={(*impersonate_mnt).clone()}
                                                on_change={on_impersonate_mnt_change}
                                                placeholder="Optional mntner override for your target ASN"
                                                disabled={*loading}
                                            />
                                        </ShellLine>
                                        <div class="autopeer-inline-actions">
                                            <ShellButton
                                                text="Impersonate This ASN"
                                                onclick={on_impersonate}
                                                disabled={*loading || impersonate_asn.trim().is_empty()}
                                            />
                                            if auth_summary.as_ref().map(|session| session.asn.as_str()) != Some(host_session.asn.as_str()) {
                                                <ShellButton
                                                    text="Return To Host ASN"
                                                    onclick={on_return_to_host}
                                                    disabled={*loading}
                                                />
                                            }
                                        </div>
                                    </div>
                                </article>
                            }

                            if let Some(operation_status) = &*operation {
                                <article class="peering-card autopeer-panel autopeer-panel--compact autopeer-status-card">
                                    <div class="autopeer-panel-header">
                                        <p class="autopeer-panel-kicker">{"Current Operation"}</p>
                                        <h3 class="autopeer-panel-title">
                                            {format!("{} {}", operation_status.kind.label(), operation_status.node)}
                                        </h3>
                                        <span class="autopeer-status-pill">{operation_status.state.label()}</span>
                                        if let Some(message) = &operation_status.message {
                                            <p class="text-secondary">{message}</p>
                                        }
                                    </div>
                                    {render_operation_progress(operation_status)}
                                    <div class="autopeer-links">
                                        if let Some(pr_url) = &operation_status.pull_request_url {
                                            <a href={pr_url.clone()} target="_blank" rel="noreferrer">{"Open PR"}</a>
                                        }
                                        if let Some(run_url) = &operation_status.workflow_run_url {
                                            <a href={run_url.clone()} target="_blank" rel="noreferrer">{"Workflow Run"}</a>
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
                    <a href={(*autopeer_site_href).clone()} class="title-link">{"DN42 Autopeer"}</a>
                    <span class="title-footnote">
                        {"of IRIS-AS 4242421023"}
                        {" / "}
                        <a href={(*looking_glass_site_href).clone()} class="autopeer-title-nav">{"Looking Glass"}</a>
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
    use common::auto_peer::{AuthMethod, AuthMethodKind};

    use super::{
        Peer6AddressKind, autopeer_home_href_from_parts, autopeer_node_endpoint_port,
        detect_peer6_address_kind, displayed_peer_config_stage, looking_glass_href_from_parts,
        retire_button_text,
    };
    use crate::{
        controller::{configured_href, filter_supported_methods, validate_ssh_signature_input},
        store::PeerConfigStage,
    };

    #[test]
    fn derives_autopeer_home_for_shared_path_deployments() {
        assert_eq!(
            autopeer_home_href_from_parts("https:", "lg.owo.li", "/autopeer/setup"),
            "https://lg.owo.li/autopeer"
        );
    }

    #[test]
    fn derives_autopeer_home_for_dedicated_host_deployments() {
        assert_eq!(
            autopeer_home_href_from_parts("https:", "autopeer.owo.li", "/"),
            "https://autopeer.owo.li/"
        );
    }

    #[test]
    fn derives_looking_glass_url_for_shared_path_deployments() {
        assert_eq!(
            looking_glass_href_from_parts("https:", "lg.owo.li", "/autopeer/setup"),
            "https://lg.owo.li/"
        );
    }

    #[test]
    fn derives_looking_glass_url_for_dedicated_host_deployments() {
        assert_eq!(
            looking_glass_href_from_parts("https:", "autopeer.owo.li", "/"),
            "https://lg.owo.li/"
        );
    }

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
        assert_eq!(retire_button_text(false), "Retire This Session");
        assert_eq!(retire_button_text(true), "Confirm Retirement");
    }

    #[test]
    fn rejects_raw_challenge_text_in_ssh_signature_field() {
        assert_eq!(
            validate_ssh_signature_input(
                "dn42-autopeer challenge\nasn: 4242421024\nchallenge_id: example\nissued_at: 2026-04-18T12:42:04.075Z"
            ),
            Err(
                "Paste the detached SSH signature block from the command above, not the unsigned challenge text."
            ),
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
