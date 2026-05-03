use ui_components::shell::{ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect};
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use super::{
    pgp_export_command, pgp_sign_command, render_error, render_ongoing_tasks,
    render_readonly_block, ssh_sign_command,
};
use crate::{
    controller::{
        OngoingTask, PgpKeyLookup, PgpKeyLookups, default_pgp_key, selected_registry_email_target,
    },
    i18n::I18n,
    models::{AuthMethod, AuthMethodKind, UiMessage},
};

#[derive(Properties, PartialEq)]
pub struct VerifyMethodProps {
    pub i18n: I18n,
    pub loading: bool,
    pub asn: String,
    pub selected_method: Option<AuthMethod>,
    pub challenge_text: Option<String>,
    pub ssh_signature: String,
    pub on_ssh_signature_change: Callback<String>,
    pub selected_pgp_key: String,
    pub pgp_public_key: String,
    pub pgp_signed_message: String,
    pub pgp_key_lookups: PgpKeyLookups,
    pub on_pgp_key_change: Callback<String>,
    pub on_pgp_public_key_change: Callback<String>,
    pub on_pgp_signed_message_change: Callback<String>,
    pub selected_email_maintainer: String,
    pub registry_email_code: String,
    pub registry_email_sent_to: Vec<String>,
    pub on_email_maintainer_change: Callback<String>,
    pub on_registry_email_code_change: Callback<String>,
    pub on_send_registry_email: Callback<MouseEvent>,
    pub ongoing_tasks: Vec<OngoingTask>,
    pub error: Option<UiMessage>,
    pub on_verify: Callback<MouseEvent>,
    pub on_verify_back: Callback<MouseEvent>,
}

#[function_component(VerifyMethodPanel)]
pub fn verify_method_panel(props: &VerifyMethodProps) -> Html {
    let i18n = &props.i18n;

    let Some(method) = &props.selected_method else {
        return html! {
            <div class="autopeer-step">
                <ShellLine>
                    <span class="error-message">{i18n.t("verify.choose_first")}</span>
                </ShellLine>
            </div>
        };
    };

    let method_label = i18n.translate_message(&method.label);
    let verification_fields = match method.kind {
        AuthMethodKind::RegistrySsh => {
            let on_change = props.on_ssh_signature_change.clone();
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
                    if let Some(challenge) = &props.challenge_text {
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
                            value={props.ssh_signature.clone()}
                            on_change={on_change}
                            placeholder={i18n.t("verify.ssh.placeholder")}
                            disabled={props.loading}
                            multiline=true
                            rows={10}
                        />
                    </ShellLine>
                </>
            }
        }
        AuthMethodKind::RegistryPgp => {
            let on_pubkey_change = props.on_pgp_public_key_change.clone();
            let on_signed_change = props.on_pgp_signed_message_change.clone();
            let selected_key_value = if props.selected_pgp_key.is_empty() {
                default_pgp_key(method)
            } else {
                props.selected_pgp_key.clone()
            };
            let on_key_change = {
                let on_pgp_key_change = props.on_pgp_key_change.clone();
                Callback::from(move |event: Event| {
                    let select: HtmlSelectElement = event.target_unchecked_into();
                    on_pgp_key_change.emit(select.value());
                })
            };
            let lookup_for_selected = props
                .pgp_key_lookups
                .get(selected_key_value.trim())
                .cloned();

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
                                {for method.pgp_fingerprints.iter().map(|fingerprint| {
                                    let status_suffix = match props.pgp_key_lookups.get(fingerprint) {
                                        Some(PgpKeyLookup::Loading) => " — \u{2026}",
                                        Some(PgpKeyLookup::Found { .. }) => " \u{2713}",
                                        Some(PgpKeyLookup::NotFound) => " \u{2717}",
                                        None => "",
                                    };
                                    html! {
                                        <option value={fingerprint.clone()}>
                                            {format!("{}{}", fingerprint, status_suffix)}
                                        </option>
                                    }
                                })}
                            </ShellSelect>
                        </ShellLine>
                    }
                    if let Some(challenge) = &props.challenge_text {
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
                            value={props.pgp_signed_message.clone()}
                            on_change={on_signed_change}
                            placeholder={i18n.t("verify.pgp.signed_placeholder")}
                            disabled={props.loading}
                            multiline=true
                            rows={12}
                        />
                    </ShellLine>
                    {render_pgp_public_key_section(
                        i18n,
                        props.loading,
                        &selected_key_value,
                        &props.pgp_public_key,
                        on_pubkey_change,
                        lookup_for_selected.as_ref(),
                    )}
                </>
            }
        }
        AuthMethodKind::RegistryEmail => {
            let selected_target =
                selected_registry_email_target(method, props.selected_email_maintainer.as_str());
            let selected_target_value = selected_target
                .map(|target| target.maintainer.clone())
                .unwrap_or_else(|| props.selected_email_maintainer.clone());
            let on_target_change = {
                let on_email_maintainer_change = props.on_email_maintainer_change.clone();
                Callback::from(move |event: Event| {
                    let select: HtmlSelectElement = event.target_unchecked_into();
                    on_email_maintainer_change.emit(select.value());
                })
            };
            let on_code_change = props.on_registry_email_code_change.clone();
            let send_button_text = if props.registry_email_sent_to.is_empty() {
                i18n.t("action.send_signin_link")
            } else {
                i18n.t("action.resend_signin_link")
            };
            let selected_target_emails = selected_target.map(|target| target.emails.join(", "));

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
                    if !props.registry_email_sent_to.is_empty() {
                        <ShellLine>
                            <span class="text-secondary">
                                {{
                                    let emails = props.registry_email_sent_to.join(", ");
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
                            onclick={props.on_send_registry_email.clone()}
                            disabled={props.loading || selected_target.is_none()}
                        />
                    </ShellLine>
                    <ShellLine>
                        <ShellPrompt>{i18n.t("prompt.code")}</ShellPrompt>
                        {" "}{i18n.t("verify.email.code_prompt")}
                    </ShellLine>
                    <ShellLine>
                        <ShellInput
                            value={props.registry_email_code.clone()}
                            on_change={on_code_change}
                            placeholder={i18n.t("verify.email.code_placeholder")}
                            disabled={props.loading}
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
                        ("asn", props.asn.as_str()),
                    ],
                )}
            </ShellLine>
            {verification_fields}
            {render_ongoing_tasks(i18n, &props.ongoing_tasks)}
            {render_error(i18n, &props.error)}
            <ShellLine>
                <ShellButton
                    text={i18n.t("action.back")}
                    onclick={props.on_verify_back.clone()}
                    disabled={props.loading}
                />
                {" "}
                <ShellButton text={verify_button_text} onclick={props.on_verify.clone()} disabled={props.loading} />
            </ShellLine>
        </div>
    }
}

fn render_pgp_public_key_section(
    i18n: &I18n,
    loading: bool,
    selected_fingerprint: &str,
    manual_public_key: &str,
    on_pubkey_change: Callback<String>,
    lookup: Option<&PgpKeyLookup>,
) -> Html {
    match lookup {
        Some(PgpKeyLookup::Loading) => html! {
            <ShellLine>
                <span class="text-secondary">
                    {i18n.t("verify.pgp.lookup.searching")}
                </span>
            </ShellLine>
        },
        Some(PgpKeyLookup::Found {
            public_key, source, ..
        }) => {
            let label = match source {
                Some(source) if !source.is_empty() => i18n.translate_params(
                    "verify.pgp.lookup.found_from",
                    &[("source", source.as_str())],
                ),
                _ => i18n.t("verify.pgp.lookup.found").to_string(),
            };
            html! {
                <>
                    <ShellLine>
                        <ShellPrompt>{i18n.t("prompt.pubkey")}</ShellPrompt>
                        {" "}{label}
                    </ShellLine>
                    <ShellLine>
                        <ShellInput
                            value={public_key.clone()}
                            on_change={Callback::noop()}
                            placeholder={i18n.t("verify.pgp.pubkey_placeholder")}
                            disabled=true
                            multiline=true
                            rows={8}
                        />
                    </ShellLine>
                </>
            }
        }
        _ => html! {
            <>
                {render_readonly_block(
                    i18n.t("verify.pgp.export_label"),
                    pgp_export_command(selected_fingerprint),
                )}
                <ShellLine>
                    <ShellPrompt>{i18n.t("prompt.pubkey")}</ShellPrompt>
                    {" "}{i18n.t("verify.pgp.pubkey_paste_prompt")}
                </ShellLine>
                <ShellLine>
                    <ShellInput
                        value={manual_public_key.to_string()}
                        on_change={on_pubkey_change}
                        placeholder={i18n.t("verify.pgp.pubkey_placeholder")}
                        disabled={loading}
                        multiline=true
                        rows={8}
                    />
                </ShellLine>
            </>
        },
    }
}
