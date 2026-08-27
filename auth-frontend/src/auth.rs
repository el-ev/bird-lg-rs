use components::shell::ShellInput;
use dn42_auth_client::{
    fragment::{decode_auth_session, encode_auth_session},
    models::{AuthMethod, AuthMethodKind, AuthSessionResponse, AuthStartResponse},
    service::{self, RuntimeConfig},
};
use wasm_bindgen_futures::spawn_local;
use web_sys::UrlSearchParams;
use yew::{AttrValue, prelude::*};

use crate::{
    components::{ShellButton, ShellLine, ShellPrompt, ShellSelect},
    i18n::use_i18n,
};

const API_BASE: &str = "https://dn42-auth.owo.li";
fn get_return_to() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|s| UrlSearchParams::new_with_str(&s).ok())
        .and_then(|params| params.get("return_to"))
        .or_else(|| web_sys::window().and_then(|w| w.location().origin().ok()))
        .unwrap_or_default()
}

fn is_self_return(return_to: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .is_some_and(|origin| origin == return_to)
}

fn get_hash_param(name: &str) -> Option<String> {
    web_sys::window().and_then(|window| {
        let hash = window.location().hash().ok()?;
        let query = hash.strip_prefix('#').unwrap_or(&hash);
        let params = UrlSearchParams::new_with_str(query).ok()?;
        params.get(name)
    })
}

fn clean_url() {
    if let Some(window) = web_sys::window() {
        let path = window.location().pathname().unwrap_or_else(|_| "/".into());
        let search = window.location().search().unwrap_or_default();
        let clean = format!("{path}{search}");
        let _ = window
            .history()
            .and_then(|h| h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&clean)));
    }
}

fn redirect_with_session(return_to: &str, session: &AuthSessionResponse) {
    let encoded = encode_auth_session(session).unwrap_or_default();
    let url = format!("{}#auth_session={}", return_to, encoded);
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(&url);
    }
}

fn show_invalid_return_to(error: &UseStateHandle<Option<String>>) {
    error.set(Some("error.auth.return_to.invalid".to_string()));
}

fn finish_auth(
    step: &UseStateHandle<Step>,
    error: &UseStateHandle<Option<String>>,
    allowed_return_urls: &[String],
    return_to: &str,
    session: AuthSessionResponse,
) {
    if is_self_return(return_to) {
        step.set(Step::Success {
            session,
            redirected: false,
        });
    } else if allowed_return_urls.iter().any(|allowed| allowed == return_to) {
        redirect_with_session(return_to, &session);
        step.set(Step::Success {
            session,
            redirected: true,
        });
    } else {
        show_invalid_return_to(error);
        step.set(Step::EnterAsn);
    }
}

fn redirect_to(url: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(url);
    }
}

fn format_error(msg: &dn42_auth_client::models::UiMessage) -> String {
    if msg.params.is_empty() {
        msg.key.clone()
    } else {
        let mut s = msg.key.clone();
        for (k, v) in &msg.params {
            s = s.replace(&format!("{{{k}}}"), v);
        }
        s
    }
}

fn render_readonly_block(label: &str, content: &str) -> Html {
    let rows = content.lines().count().max(1);
    let value = content.to_string();
    let on_focus = Callback::from(move |event: FocusEvent| {
        let target: web_sys::HtmlTextAreaElement = event.target_unchecked_into();
        target.select();
    });
    let on_click = Callback::from(move |event: MouseEvent| {
        let target: web_sys::HtmlTextAreaElement = event.target_unchecked_into();
        target.select();
    });

    html! {
        <div class="auth-command-block">
            <div class="auth-command-label">{format!("# {label}")}</div>
            <textarea
                class="auth-command-textarea"
                readonly=true
                spellcheck="false"
                rows={rows.to_string()}
                value={value}
                onfocus={on_focus}
                onclick={on_click}
            />
        </div>
    }
}

#[derive(Clone, PartialEq)]
enum Step {
    Loading,
    EnterAsn,
    SelectMethod {
        challenge: AuthStartResponse,
    },
    VerifySsh {
        challenge: AuthStartResponse,
        method: AuthMethod,
    },
    VerifyPgp {
        challenge: AuthStartResponse,
        method: AuthMethod,
    },
    VerifyEmail {
        challenge: AuthStartResponse,
        method: AuthMethod,
    },
    Success {
        session: AuthSessionResponse,
        redirected: bool,
    },
}

#[function_component(AuthFlow)]
pub fn auth_flow() -> Html {
    let step = use_state(|| Step::Loading);
    let asn_input = use_state(String::new);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let loading_message = use_state(|| None::<&'static str>);
    let ssh_signature = use_state(String::new);
    let pgp_signed_message = use_state(String::new);
    let pgp_public_key = use_state(String::new);
    let selected_pgp_fp = use_state(String::new);
    let email_code = use_state(String::new);
    let email_sent_to = use_state(Vec::<String>::new);
    let selected_email_mnt = use_state(String::new);
    let return_to = use_state(get_return_to);
    let config = use_state(RuntimeConfig::default);
    let i18n = use_i18n();

    {
        let i18n = i18n.clone();
        use_effect_with((), move |_| {
            if let Some(lang) = web_sys::window()
                .and_then(|w| w.location().search().ok())
                .and_then(|s| UrlSearchParams::new_with_str(&s).ok())
                .and_then(|p| p.get("lang"))
                && let Some(locale) = crate::i18n::Locale::from_code(&lang) {
                    i18n.set_locale(locale);
                }
            || ()
        });
    }

    {
        let step = step.clone();
        let error = error.clone();
        let return_to = return_to.clone();
        let config = config.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let runtime_config = service::load_runtime_config().await.unwrap_or_default();
                let allowed_return_urls = runtime_config.allowed_return_urls.clone();
                config.set(runtime_config);

                if let Some(encoded) = get_hash_param("auth_session") {
                    clean_url();
                    if let Some(session) = decode_auth_session(&encoded) {
                        let return_to_val = (*return_to).clone();
                        finish_auth(&step, &error, &allowed_return_urls, &return_to_val, session);
                    } else {
                        error.set(Some("error.auth.session.invalid".to_string()));
                        step.set(Step::EnterAsn);
                    }
                } else if let Some(oidc_error) = get_hash_param("oidc_error") {
                    clean_url();
                    error.set(Some(oidc_error));
                    step.set(Step::EnterAsn);
                } else if let Some(email_error) = get_hash_param("email_error") {
                    clean_url();
                    error.set(Some(email_error));
                    step.set(Step::EnterAsn);
                } else {
                    step.set(Step::EnterAsn);
                }
            });
            || ()
        });
    }

    let submit_asn = {
        let asn_input = asn_input.clone();
        let step = step.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();

        Callback::from(move |_: ()| {
            let asn_value = asn_input.trim().to_string();
            if asn_value.is_empty() {
                error.set(Some("error.auth.asn.required".to_string()));
                return;
            }

            loading.set(true);
            loading_message.set(Some("auth.finding_methods"));
            error.set(None);

            let step = step.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();

            spawn_local(async move {
                match service::start_auth(API_BASE, &asn_value).await {
                    Ok(challenge) => {
                        step.set(Step::SelectMethod { challenge });
                    }
                    Err(msg) => {
                        error.set(Some(format_error(&msg)));
                    }
                }
                loading.set(false);
                loading_message.set(None);
            });
        })
    };

    let on_asn_change = {
        let asn_input = asn_input.clone();
        Callback::from(move |v: String| asn_input.set(v))
    };

    let on_submit_asn = {
        let submit_asn = submit_asn.clone();
        Callback::from(move |_: MouseEvent| submit_asn.emit(()))
    };

    let on_asn_keydown = {
        let submit_asn = submit_asn.clone();
        let loading = loading.clone();
        let asn_input = asn_input.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !*loading && !asn_input.trim().is_empty() {
                e.prevent_default();
                submit_asn.emit(());
            }
        })
    };

    let on_select_method = {
        let step = step.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let ssh_signature = ssh_signature.clone();
        let pgp_signed_message = pgp_signed_message.clone();
        let pgp_public_key = pgp_public_key.clone();
        let selected_pgp_fp = selected_pgp_fp.clone();
        let email_code = email_code.clone();
        let email_sent_to = email_sent_to.clone();
        let selected_email_mnt = selected_email_mnt.clone();
        let return_to = return_to.clone();

        Callback::from(
            move |(challenge, method): (AuthStartResponse, AuthMethod)| {
                error.set(None);
                ssh_signature.set(String::new());
                pgp_signed_message.set(String::new());
                pgp_public_key.set(String::new());
                selected_pgp_fp.set(String::new());
                email_code.set(String::new());
                email_sent_to.set(Vec::new());
                let default_mnt = method
                    .email_targets
                    .first()
                    .map(|t| t.maintainer.clone())
                    .unwrap_or_default();
                selected_email_mnt.set(default_mnt);

                match method.kind {
                    AuthMethodKind::Oidc => {
                        let Some(provider) = method.provider.clone() else {
                            error.set(Some("error.auth.oidc.no_provider".to_string()));
                            return;
                        };
                        loading.set(true);
                        loading_message.set(Some("auth.oidc_redirecting"));
                        error.set(None);
                        let error = error.clone();
                        let loading = loading.clone();
                        let loading_message = loading_message.clone();
                        let challenge_id = challenge.challenge_id.clone();
                        let return_to = (*return_to).clone();
                        spawn_local(async move {
                            match service::start_oidc(
                                API_BASE,
                                &provider,
                                Some(&challenge_id),
                                Some(&return_to),
                            )
                            .await
                            {
                                Ok(resp) => redirect_to(&resp.authorization_url),
                                Err(msg) => {
                                    error.set(Some(format_error(&msg)));
                                    loading.set(false);
                                    loading_message.set(None);
                                }
                            }
                        });
                    }
                    AuthMethodKind::RegistrySsh => {
                        step.set(Step::VerifySsh { challenge, method });
                    }
                    AuthMethodKind::RegistryPgp => {
                        step.set(Step::VerifyPgp { challenge, method });
                    }
                    AuthMethodKind::RegistryEmail => {
                        step.set(Step::VerifyEmail { challenge, method });
                    }
                    AuthMethodKind::HostImpersonation => {
                        error.set(Some(
                            "error.auth.host_impersonation.unsupported".to_string(),
                        ));
                    }
                }
            },
        )
    };

    let on_back_to_select = {
        let step = step.clone();
        let error = error.clone();
        Callback::from(move |challenge: AuthStartResponse| {
            error.set(None);
            step.set(Step::SelectMethod { challenge });
        })
    };

    let on_back_to_asn = {
        let step = step.clone();
        let error = error.clone();
        Callback::from(move |_: MouseEvent| {
            error.set(None);
            step.set(Step::EnterAsn);
        })
    };

    let on_verify_ssh = {
        let step = step.clone();
        let error = error.clone();
        let loading = loading.clone();
        let ssh_signature = ssh_signature.clone();
        let return_to = return_to.clone();
        let config_for_ssh = config.clone();

        Callback::from(move |(challenge_id, _method): (String, AuthMethod)| {
            let sig = ssh_signature.trim().to_string();
            if !sig.contains("-----BEGIN SSH SIGNATURE-----")
                || !sig.contains("-----END SSH SIGNATURE-----")
            {
                if sig.contains("dn42") {
                    error.set(Some(
                        "error.auth.ssh.paste_signed_not_challenge".to_string(),
                    ));
                } else {
                    error.set(Some("error.auth.ssh.paste_complete_block".to_string()));
                }
                return;
            }
            loading.set(true);
            error.set(None);
            let step = step.clone();
            let error = error.clone();
            let loading = loading.clone();
            let return_to_val = (*return_to).clone();
            let config = (*config_for_ssh).clone();
            spawn_local(async move {
                match service::verify_registry_ssh(API_BASE, &challenge_id, &sig).await {
                    Ok(session) => {
                        finish_auth(
                            &step,
                            &error,
                            &config.allowed_return_urls,
                            &return_to_val,
                            session,
                        );
                    }
                    Err(msg) => error.set(Some(format_error(&msg))),
                }
                loading.set(false);
            });
        })
    };

    let on_verify_pgp = {
        let step = step.clone();
        let error = error.clone();
        let loading = loading.clone();
        let pgp_signed_message = pgp_signed_message.clone();
        let pgp_public_key = pgp_public_key.clone();
        let return_to = return_to.clone();
        let config_for_pgp = config.clone();

        Callback::from(move |(challenge_id, _method): (String, AuthMethod)| {
            let signed = pgp_signed_message.trim().to_string();
            let pubkey = pgp_public_key.trim().to_string();
            if signed.is_empty() {
                error.set(Some("error.auth.pgp.paste_signed".to_string()));
                return;
            }
            loading.set(true);
            error.set(None);
            let step = step.clone();
            let error = error.clone();
            let loading = loading.clone();
            let return_to_val = (*return_to).clone();
            let config = (*config_for_pgp).clone();
            spawn_local(async move {
                match service::verify_registry_pgp(API_BASE, &challenge_id, &pubkey, &signed).await
                {
                    Ok(session) => {
                        finish_auth(
                            &step,
                            &error,
                            &config.allowed_return_urls,
                            &return_to_val,
                            session,
                        );
                    }
                    Err(msg) => error.set(Some(format_error(&msg))),
                }
                loading.set(false);
            });
        })
    };

    let on_send_email = {
        let error = error.clone();
        let loading = loading.clone();
        let email_sent_to = email_sent_to.clone();
        let selected_email_mnt = selected_email_mnt.clone();

        Callback::from(move |(challenge_id, method): (String, AuthMethod)| {
            let effective_mnt = if selected_email_mnt.trim().is_empty() {
                method.email_targets.first().map(|t| t.maintainer.clone())
            } else {
                Some((*selected_email_mnt).clone())
            };
            let Some(mnt) = effective_mnt else {
                error.set(Some("error.auth.email.no_targets".to_string()));
                return;
            };
            loading.set(true);
            error.set(None);
            let error = error.clone();
            let loading = loading.clone();
            let email_sent_to = email_sent_to.clone();
            spawn_local(async move {
                match service::send_registry_email(API_BASE, &challenge_id, Some(&mnt)).await {
                    Ok(resp) => {
                        email_sent_to.set(resp.emails);
                        error.set(None);
                    }
                    Err(msg) => error.set(Some(format_error(&msg))),
                }
                loading.set(false);
            });
        })
    };

    let on_verify_email = {
        let step = step.clone();
        let error = error.clone();
        let loading = loading.clone();
        let email_code = email_code.clone();
        let return_to = return_to.clone();
        let config_for_email = config.clone();

        Callback::from(move |(challenge_id, _method): (String, AuthMethod)| {
            let code = email_code.trim().to_string();
            if code.is_empty() {
                error.set(Some("error.auth.email.code_empty".to_string()));
                return;
            }
            loading.set(true);
            error.set(None);
            let step = step.clone();
            let error = error.clone();
            let loading = loading.clone();
            let return_to_val = (*return_to).clone();
            let config = (*config_for_email).clone();
            spawn_local(async move {
                match service::verify_registry_email(API_BASE, &challenge_id, &code).await {
                    Ok(session) => {
                        finish_auth(
                            &step,
                            &error,
                            &config.allowed_return_urls,
                            &return_to_val,
                            session,
                        );
                    }
                    Err(msg) => error.set(Some(format_error(&msg))),
                }
                loading.set(false);
            });
        })
    };

    match (*step).clone() {
        Step::Loading => html! {
            <div class="auth-section">
                <ShellLine>
                    <ShellPrompt text={i18n.t("prompt.status")} />
                    {format!(" {}", i18n.t("auth.loading"))}
                </ShellLine>
            </div>
        },

        Step::EnterAsn => {
            let oidc_methods = config.oidc_methods.clone();
            html! {
                <div class="auth-section">
                    <ShellLine>
                        {i18n.t("auth.prompt")}
                    </ShellLine>
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.asn")} />
                        {" "}
                        <ShellInput
                            value={AttrValue::from((*asn_input).clone())}
                            on_change={on_asn_change}
                            on_keydown={on_asn_keydown.clone()}
                            placeholder={"424242xxxx"}
                            disabled={*loading}
                        />
                    </ShellLine>
                    <ShellLine>
                        <ShellButton
                            text={i18n.t("action.find_methods")}
                            onclick={on_submit_asn}
                            disabled={*loading || asn_input.trim().is_empty()}
                        />
                    </ShellLine>
                    if !oidc_methods.is_empty() {
                        <p>
                            {i18n.t("auth.oidc_alt")}
                        </p>
                        { for oidc_methods.iter().filter_map(|method| {
                            let provider = method.provider.as_deref()?;
                            let method_label = i18n.translate_message(&method.label);
                            let label = i18n.t("auth.continue_with").replace("{provider}", &method_label);
                            let desc = i18n.translate_message(&method.description);
                            let provider = provider.to_string();
                            let return_to = (*return_to).clone();
                            let loading_for_click = loading.clone();
                            let loading_message_for_click = loading_message.clone();
                            let error = error.clone();
                            let onclick = Callback::from(move |_: MouseEvent| {
                                error.set(None);
                                loading_for_click.set(true);
                                loading_message_for_click.set(Some("auth.oidc_redirecting"));
                                let provider = provider.clone();
                                let return_to = return_to.clone();
                                let error = error.clone();
                                let loading = loading_for_click.clone();
                                let loading_message = loading_message_for_click.clone();
                                spawn_local(async move {
                                    match service::start_oidc(
                                        API_BASE,
                                        &provider,
                                        None,
                                        Some(&return_to),
                                    )
                                    .await
                                    {
                                        Ok(resp) => redirect_to(&resp.authorization_url),
                                        Err(msg) => {
                                            error.set(Some(format_error(&msg)));
                                            loading.set(false);
                                            loading_message.set(None);
                                        }
                                    }
                                });
                            });
                            Some(html! {
                                <ShellLine>
                                    <ShellButton text={label} onclick={onclick} disabled={*loading} />
                                    <span class="auth-method-inline-desc">{format!(" - {desc}")}</span>
                                </ShellLine>
                            })
                        }) }
                    }
                    if let Some(msg) = *loading_message {
                        <ShellLine>
                            <span class="status-message">{i18n.t(msg)}</span>
                        </ShellLine>
                    }
                    if let Some(err) = (*error).clone() {
                        <ShellLine>
                            <span class="status-message status-message--error">{i18n.translate_owned(&err)}</span>
                        </ShellLine>
                    }
                </div>
            }
        }

        Step::SelectMethod { challenge } => {
            let methods = challenge.methods.clone();
            let on_back = on_back_to_asn.clone();

            html! {
                <div class="auth-section">
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.auth")} />
                        {format!(" {}", i18n.t("auth.asn_methods").replace("{asn}", &challenge.asn))}
                    </ShellLine>
                    { for methods.iter().map(|method| {
                        let method_clone = method.clone();
                        let challenge_clone = challenge.clone();
                        let on_select = on_select_method.clone();
                        let label = i18n.translate_message(&method.label);
                        let desc = i18n.translate_message(&method.description);
                        let onclick = Callback::from(move |_: MouseEvent| {
                            on_select.emit((challenge_clone.clone(), method_clone.clone()));
                        });
                        html! {
                            <ShellLine>
                                <ShellButton text={label} onclick={onclick} disabled={*loading} />
                                <span class="auth-method-inline-desc">{format!(" - {desc}")}</span>
                            </ShellLine>
                        }
                    }) }
                    if let Some(msg) = *loading_message {
                        <ShellLine>
                            <span class="status-message">{i18n.t(msg)}</span>
                        </ShellLine>
                    }
                    if let Some(err) = (*error).clone() {
                        <ShellLine>
                            <span class="status-message status-message--error">{i18n.translate_owned(&err)}</span>
                        </ShellLine>
                    }
                    <ShellLine>
                        <ShellButton text={i18n.t("action.back")} onclick={on_back} disabled={*loading} />
                    </ShellLine>
                </div>
            }
        }

        Step::VerifySsh { challenge, method } => {
            let challenge_id = challenge.challenge_id.clone();
            let challenge_text = challenge.challenge_text.clone();
            let method_label = i18n.translate_message(&method.label);

            let ssh_cmd = format!(
                "ssh-keygen -Y sign -f <PRIVATE_KEY_PATH> -n file <<'EOF'\n{challenge_text}\nEOF"
            );

            let on_verify = {
                let on_verify_ssh = on_verify_ssh.clone();
                let method = method.clone();
                let challenge_id = challenge_id.clone();
                Callback::from(move |_: MouseEvent| {
                    on_verify_ssh.emit((challenge_id.clone(), method.clone()));
                })
            };

            let on_back = {
                let challenge = challenge.clone();
                let on_back_to_select = on_back_to_select.clone();
                Callback::from(move |_: MouseEvent| {
                    on_back_to_select.emit(challenge.clone());
                })
            };

            let on_sig_change = {
                let ssh_signature = ssh_signature.clone();
                Callback::from(move |v: String| ssh_signature.set(v))
            };

            html! {
                <div class="auth-section">
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.auth")} />
                        {format!(" {} for AS{}", method_label, challenge.asn)}
                    </ShellLine>
                    if method.ssh_fingerprints.len() == 1 {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.key")} />
                            {format!(" {}", method.ssh_fingerprints[0])}
                        </ShellLine>
                    } else if method.ssh_fingerprints.len() > 1 {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.key")} />
                            {format!(" {}", method.ssh_fingerprints.join(", "))}
                        </ShellLine>
                    }
                    {render_readonly_block(i18n.t("block.sign_command"), &ssh_cmd)}
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.signature")} />
                        {format!(" {}", i18n.t("ssh.paste_prompt"))}
                    </ShellLine>
                    <ShellLine>
                        <ShellInput
                            value={AttrValue::from((*ssh_signature).clone())}
                            on_change={on_sig_change}
                            placeholder={"-----BEGIN SSH SIGNATURE-----"}
                            disabled={*loading}
                            multiline=true
                            rows={10}
                        />
                    </ShellLine>
                    if let Some(err) = (*error).clone() {
                        <ShellLine>
                            <span class="status-message status-message--error">{i18n.translate_owned(&err)}</span>
                        </ShellLine>
                    }
                    <ShellLine>
                        <ShellButton text={i18n.t("action.back")} onclick={on_back} disabled={*loading} />
                        {" "}
                        <ShellButton text={i18n.t("action.verify")} onclick={on_verify} disabled={*loading} />
                    </ShellLine>
                </div>
            }
        }

        Step::VerifyPgp { challenge, method } => {
            let challenge_id = challenge.challenge_id.clone();
            let challenge_text = challenge.challenge_text.clone();
            let method_label = i18n.translate_message(&method.label);

            let fingerprints = method.pgp_fingerprints.clone();
            let fingerprint = if (*selected_pgp_fp).is_empty() {
                fingerprints.first().cloned().unwrap_or_default()
            } else {
                (*selected_pgp_fp).clone()
            };
            let pgp_sign_cmd = if fingerprint.is_empty() {
                format!("gpg --armor --clearsign <<'EOF'\n{challenge_text}\nEOF")
            } else {
                format!(
                    "gpg --armor --local-user {fingerprint} --clearsign <<'EOF'\n{challenge_text}\nEOF"
                )
            };
            let pgp_export_cmd = if fingerprint.is_empty() {
                "gpg --armor --export <KEYID_OR_FINGERPRINT>".to_string()
            } else {
                format!("gpg --armor --export {fingerprint}")
            };

            let on_verify = {
                let on_verify_pgp = on_verify_pgp.clone();
                let method = method.clone();
                let challenge_id = challenge_id.clone();
                Callback::from(move |_: MouseEvent| {
                    on_verify_pgp.emit((challenge_id.clone(), method.clone()));
                })
            };

            let on_back = {
                let challenge = challenge.clone();
                let on_back_to_select = on_back_to_select.clone();
                Callback::from(move |_: MouseEvent| {
                    on_back_to_select.emit(challenge.clone());
                })
            };

            let on_signed_change = {
                let pgp_signed_message = pgp_signed_message.clone();
                Callback::from(move |v: String| pgp_signed_message.set(v))
            };

            let on_pubkey_change = {
                let pgp_public_key = pgp_public_key.clone();
                Callback::from(move |v: String| pgp_public_key.set(v))
            };

            let on_fp_change = {
                let selected_pgp_fp = selected_pgp_fp.clone();
                Callback::from(move |v: String| selected_pgp_fp.set(v))
            };

            html! {
                <div class="auth-section">
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.auth")} />
                        {format!(" {} for AS{}", method_label, challenge.asn)}
                    </ShellLine>
                    if fingerprints.len() == 1 {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.key")} />
                            {format!(" {}", fingerprints[0])}
                        </ShellLine>
                    } else if fingerprints.len() > 1 {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.key")} />
                            {" "}
                            <ShellSelect
                                value={fingerprint.clone()}
                                on_change={on_fp_change}
                                options={fingerprints.clone()}
                                disabled={*loading}
                            />
                        </ShellLine>
                    }
                    {render_readonly_block(i18n.t("block.challenge"), &challenge_text)}
                    {render_readonly_block(i18n.t("block.sign_command"), &pgp_sign_cmd)}
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.signed")} />
                        {format!(" {}", i18n.t("pgp.paste_signed"))}
                    </ShellLine>
                    <ShellLine>
                        <ShellInput
                            value={AttrValue::from((*pgp_signed_message).clone())}
                            on_change={on_signed_change}
                            placeholder={"-----BEGIN PGP SIGNED MESSAGE-----"}
                            disabled={*loading}
                            multiline=true
                            rows={12}
                        />
                    </ShellLine>
                    {render_readonly_block(i18n.t("block.export_command"), &pgp_export_cmd)}
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.pubkey")} />
                        {format!(" {}", i18n.t("pgp.paste_pubkey"))}
                    </ShellLine>
                    <ShellLine>
                        <ShellInput
                            value={AttrValue::from((*pgp_public_key).clone())}
                            on_change={on_pubkey_change}
                            placeholder={"-----BEGIN PGP PUBLIC KEY BLOCK-----"}
                            disabled={*loading}
                            multiline=true
                            rows={8}
                        />
                    </ShellLine>
                    if let Some(err) = (*error).clone() {
                        <ShellLine>
                            <span class="status-message status-message--error">{i18n.translate_owned(&err)}</span>
                        </ShellLine>
                    }
                    <ShellLine>
                        <ShellButton text={i18n.t("action.back")} onclick={on_back} disabled={*loading} />
                        {" "}
                        <ShellButton text={i18n.t("action.verify")} onclick={on_verify} disabled={*loading} />
                    </ShellLine>
                </div>
            }
        }

        Step::VerifyEmail { challenge, method } => {
            let challenge_id = challenge.challenge_id.clone();

            let email_targets: Vec<String> = method
                .email_targets
                .iter()
                .map(|t| t.maintainer.clone())
                .collect();
            let show_select = email_targets.len() > 1;

            let current_mnt = if (*selected_email_mnt).is_empty() {
                email_targets.first().cloned().unwrap_or_default()
            } else {
                (*selected_email_mnt).clone()
            };

            let on_send = {
                let on_send_email = on_send_email.clone();
                let method = method.clone();
                let challenge_id = challenge_id.clone();
                Callback::from(move |_: MouseEvent| {
                    on_send_email.emit((challenge_id.clone(), method.clone()));
                })
            };

            let on_verify = {
                let on_verify_email = on_verify_email.clone();
                let method = method.clone();
                let challenge_id = challenge_id.clone();
                Callback::from(move |_: MouseEvent| {
                    on_verify_email.emit((challenge_id.clone(), method.clone()));
                })
            };

            let on_back = {
                let challenge = challenge.clone();
                let on_back_to_select = on_back_to_select.clone();
                Callback::from(move |_: MouseEvent| {
                    on_back_to_select.emit(challenge.clone());
                })
            };

            let on_mnt_change = {
                let selected_email_mnt = selected_email_mnt.clone();
                Callback::from(move |v: String| selected_email_mnt.set(v))
            };

            let on_code_change = {
                let email_code = email_code.clone();
                Callback::from(move |v: String| email_code.set(v))
            };

            let send_label = if (*email_sent_to).is_empty() {
                i18n.t("action.send_code")
            } else {
                i18n.t("action.resend_code")
            };

            let selected_target_emails: Option<String> = method
                .email_targets
                .iter()
                .find(|t| t.maintainer == current_mnt)
                .or(method.email_targets.first())
                .map(|t| t.emails.join(", "));

            html! {
                <div class="auth-section">
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.auth")} />
                        {format!(" {}", i18n.t("auth.email_verify").replace("{asn}", &challenge.asn))}
                    </ShellLine>
                    <ShellLine>
                        <span class="text-secondary">
                            {i18n.t("auth.email_intro")}
                        </span>
                    </ShellLine>
                    if show_select {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.mntner")} />
                            {" "}
                            <ShellSelect
                                value={current_mnt.clone()}
                                on_change={on_mnt_change}
                                options={email_targets.clone()}
                                disabled={*loading}
                            />
                        </ShellLine>
                    } else if !current_mnt.is_empty() {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.mntner")} />
                            {format!(" {}", i18n.t("auth.email_auth_as").replace("{mnt}", &current_mnt))}
                        </ShellLine>
                    }
                    if let Some(emails) = &selected_target_emails {
                        <ShellLine>
                            <ShellPrompt text={i18n.t("prompt.emails")} />
                            {format!(" {}", i18n.t("auth.email_send_to").replace("{emails}", emails))}
                        </ShellLine>
                    }
                    if !(*email_sent_to).is_empty() {
                        <ShellLine>
                            <span class="text-secondary">
                                {i18n.t("auth.code_sent").replace("{emails}", &(*email_sent_to).join(", "))}
                            </span>
                        </ShellLine>
                    }
                    <ShellLine>
                        <ShellButton text={send_label} onclick={on_send} disabled={*loading} />
                    </ShellLine>
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.code")} />
                        {format!(" {}", i18n.t("auth.email_code_prompt"))}
                    </ShellLine>
                    <ShellLine>
                        <ShellInput
                            value={AttrValue::from((*email_code).clone())}
                            on_change={on_code_change}
                            placeholder={"12345678"}
                            disabled={*loading}
                        />
                    </ShellLine>
                    if let Some(err) = (*error).clone() {
                        <ShellLine>
                            <span class="status-message status-message--error">{i18n.translate_owned(&err)}</span>
                        </ShellLine>
                    }
                    <ShellLine>
                        <ShellButton text={i18n.t("action.back")} onclick={on_back} disabled={*loading} />
                        {" "}
                        <ShellButton
                            text={i18n.t("action.verify_code")}
                            onclick={on_verify}
                            disabled={*loading || (*email_sent_to).is_empty()}
                        />
                    </ShellLine>
                </div>
            }
        }

        Step::Success {
            session,
            redirected,
        } => {
            html! {
                <div class="auth-section">
                    <ShellLine>
                        <ShellPrompt text={i18n.t("prompt.auth")} />
                        {format!(" {}", i18n.t("auth.authenticated").replace("{asn}", &session.asn).replace("{mnt}", &session.effective_mnt))}
                    </ShellLine>
                    <ShellLine>
                        <span class="status-message status-message--success">
                            {i18n.t(if redirected { "auth.redirecting" } else { "auth.complete_close" })}
                        </span>
                    </ShellLine>
                </div>
            }
        }
    }
}
