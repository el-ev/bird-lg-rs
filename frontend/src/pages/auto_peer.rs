use common::auto_peer::{ChallengeMethod, PeeringSession};
use yew::prelude::*;

use crate::{
    components::shell::{ShellButton, ShellInput, ShellLine, ShellPrompt},
    store::auto_peer::{AutoPeerAction, AutoPeerState, AutoPeerStep},
};

#[function_component(AutoPeerPage)]
pub fn auto_peer_page() -> Html {
    let state = use_reducer(AutoPeerState::default);

    let content = match state.step {
        AutoPeerStep::EnterAsn => render_asn_input(&state),
        AutoPeerStep::SelectChallenge => render_challenge_selection(&state),
        AutoPeerStep::VerifyPgp => render_pgp_verification(&state),
        AutoPeerStep::VerifyEmail => render_email_verification(&state),
        AutoPeerStep::ManageSessions => render_session_management(&state),
    };

    html! {
        <section class="autopeer">
            <div class="autopeer-container">
                {content}
            </div>
        </section>
    }
}

fn render_asn_input(state: &UseReducerHandle<AutoPeerState>) -> Html {
    let asn_value = state.asn.clone();
    let state_clone = state.clone();

    let on_asn_change = Callback::from(move |value: String| {
        state_clone.dispatch(AutoPeerAction::SetAsn(value));
    });

    let state_clone = state.clone();
    let on_submit = Callback::from(move |_| {
        // TODO: Send request to backend
        // For now, simulate response for testing
        state_clone.dispatch(AutoPeerAction::SetLoading(true));

        // Mock response - in real implementation, this would come from API
        let mock_response = common::auto_peer::AutoPeerResponse::InitSuccess {
            challenge_methods: vec![ChallengeMethod::Pgp, ChallengeMethod::Email],
        };
        state_clone.dispatch(AutoPeerAction::HandleInitResponse(mock_response));
    });

    html! {
        <div class="autopeer-step">
            <ShellLine>
                <ShellPrompt>{"autopeer"}</ShellPrompt>
                {" Enter your ASN to begin the peering process"}
            </ShellLine>

            <ShellLine>
                <ShellPrompt>{"asn"}</ShellPrompt>
                {" "}
                <ShellInput
                    value={asn_value}
                    on_change={on_asn_change}
                    placeholder="424242xxxx"
                    disabled={state.loading}
                />
            </ShellLine>

            if let Some(error) = &state.error {
                <ShellLine>
                    <span class="error-message">{error}</span>
                </ShellLine>
            }

            <ShellLine>
                <ShellButton
                    text="Submit"
                    onclick={on_submit}
                    disabled={state.loading || state.asn.is_empty()}
                />
            </ShellLine>
        </div>
    }
}

fn render_challenge_selection(state: &UseReducerHandle<AutoPeerState>) -> Html {
    let state_clone = state.clone();

    let on_select_pgp = Callback::from(move |_| {
        let state = state_clone.clone();
        state.dispatch(AutoPeerAction::SelectMethod(ChallengeMethod::Pgp));
        state.dispatch(AutoPeerAction::SetLoading(true));

        // Mock response
        let mock_response = common::auto_peer::AutoPeerResponse::ChallengeSelected {
            challenge_text: Some(
                "-----BEGIN PGP MESSAGE-----\nTest challenge text\n-----END PGP MESSAGE-----"
                    .to_string(),
            ),
        };
        state.dispatch(AutoPeerAction::HandleChallengeResponse(mock_response));
    });

    let state_clone = state.clone();
    let on_select_email = Callback::from(move |_| {
        let state = state_clone.clone();
        state.dispatch(AutoPeerAction::SelectMethod(ChallengeMethod::Email));
        state.dispatch(AutoPeerAction::SetLoading(true));

        // Mock response
        let mock_response = common::auto_peer::AutoPeerResponse::ChallengeSelected {
            challenge_text: None,
        };
        state.dispatch(AutoPeerAction::HandleChallengeResponse(mock_response));
    });

    html! {
        <div class="autopeer-step">
            <ShellLine>
                <ShellPrompt>{"autopeer"}</ShellPrompt>
                {format!(" ASN {} verified. Select verification method:", state.asn)}
            </ShellLine>

            <div class="autopeer-challenge-list">
                {for state.challenge_methods.iter().map(|method| {
                    match method {
                        ChallengeMethod::Pgp => html! {
                            <ShellLine>
                                <ShellButton
                                    text="PGP Signature"
                                    onclick={on_select_pgp.clone()}
                                    disabled={state.loading}
                                />
                                <span class="autopeer-method-desc">{" - Sign a challenge with your PGP key"}</span>
                            </ShellLine>
                        },
                        ChallengeMethod::Email => html! {
                            <ShellLine>
                                <ShellButton
                                    text="Email Verification"
                                    onclick={on_select_email.clone()}
                                    disabled={state.loading}
                                />
                                <span class="autopeer-method-desc">{" - Receive a code via email"}</span>
                            </ShellLine>
                        },
                    }
                })}
            </div>

            if let Some(error) = &state.error {
                <ShellLine>
                    <span class="error-message">{error}</span>
                </ShellLine>
            }
        </div>
    }
}

#[function_component(PgpVerification)]
fn pgp_verification(props: &PgpVerificationProps) -> Html {
    let pubkey = use_state(String::new);
    let signature = use_state(String::new);

    let pubkey_value = (*pubkey).clone();
    let pubkey_clone = pubkey.clone();
    let on_pubkey_change = Callback::from(move |value: String| {
        pubkey_clone.set(value);
    });

    let signature_value = (*signature).clone();
    let signature_clone = signature.clone();
    let on_signature_change = Callback::from(move |value: String| {
        signature_clone.set(value);
    });

    let state_clone = props.state.clone();
    let on_submit = Callback::from(move |_| {
        let state = state_clone.clone();
        state.dispatch(AutoPeerAction::SetLoading(true));

        // Mock response
        let mock_response = common::auto_peer::AutoPeerResponse::VerifySuccess {
            credential: "mock_credential_token".to_string(),
            sessions: vec![PeeringSession {
                id: Some("session1".to_string()),
                name: "Example Peer 1".to_string(),
                ipv4: Some("172.20.0.1".to_string()),
                ipv6: Some("fd00::1".to_string()),
                endpoint: "peer1.example.com:51820".to_string(),
                comment: Some("Primary peering".to_string()),
            }],
        };
        state.dispatch(AutoPeerAction::HandleVerifyResponse(mock_response));
    });

    html! {
        <div class="autopeer-step">
            <ShellLine>
                <ShellPrompt>{"pgp"}</ShellPrompt>
                {" Sign the following challenge text:"}
            </ShellLine>

            if let Some(challenge) = &props.state.challenge_text {
                <div class="autopeer-challenge-text">
                    <pre>{challenge}</pre>
                </div>
            }

            <ShellLine>
                <ShellPrompt>{"pubkey"}</ShellPrompt>
                {" "}
                <ShellInput
                    value={pubkey_value.clone()}
                    on_change={on_pubkey_change}
                    placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----"
                    disabled={props.state.loading}
                />
            </ShellLine>

            <ShellLine>
                <ShellPrompt>{"signature"}</ShellPrompt>
                {" "}
                <ShellInput
                    value={signature_value.clone()}
                    on_change={on_signature_change}
                    placeholder="-----BEGIN PGP SIGNED MESSAGE-----"
                    disabled={props.state.loading}
                />
            </ShellLine>

            if let Some(error) = &props.state.error {
                <ShellLine>
                    <span class="error-message">{error}</span>
                </ShellLine>
            }

            <ShellLine>
                <ShellButton
                    text="Verify"
                    onclick={on_submit}
                    disabled={props.state.loading || pubkey_value.is_empty() || signature_value.is_empty()}
                />
            </ShellLine>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct PgpVerificationProps {
    state: UseReducerHandle<AutoPeerState>,
}

fn render_pgp_verification(state: &UseReducerHandle<AutoPeerState>) -> Html {
    html! {
        <PgpVerification state={state.clone()} />
    }
}

#[function_component(EmailVerification)]
fn email_verification(props: &EmailVerificationProps) -> Html {
    let code = use_state(String::new);

    let code_value = (*code).clone();
    let code_clone = code.clone();
    let on_code_change = Callback::from(move |value: String| {
        code_clone.set(value);
    });

    let state_clone = props.state.clone();
    let on_submit = Callback::from(move |_| {
        let state = state_clone.clone();
        state.dispatch(AutoPeerAction::SetLoading(true));

        let mock_response = common::auto_peer::AutoPeerResponse::VerifySuccess {
            credential: "mock_credential_token".to_string(),
            sessions: vec![],
        };
        state.dispatch(AutoPeerAction::HandleVerifyResponse(mock_response));
    });

    html! {
        <div class="autopeer-step">
            <ShellLine>
                <ShellPrompt>{"email"}</ShellPrompt>
                {" A verification code has been sent to your registered email address."}
            </ShellLine>

            <ShellLine>
                <ShellPrompt>{"code"}</ShellPrompt>
                {" "}
                <ShellInput
                    value={code_value.clone()}
                    on_change={on_code_change}
                    placeholder="Enter verification code"
                    disabled={props.state.loading}
                />
            </ShellLine>

            if let Some(error) = &props.state.error {
                <ShellLine>
                    <span class="error-message">{error}</span>
                </ShellLine>
            }

            <ShellLine>
                <ShellButton
                    text="Verify"
                    onclick={on_submit}
                    disabled={props.state.loading || code_value.is_empty()}
                />
            </ShellLine>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct EmailVerificationProps {
    state: UseReducerHandle<AutoPeerState>,
}

fn render_email_verification(state: &UseReducerHandle<AutoPeerState>) -> Html {
    html! {
        <EmailVerification state={state.clone()} />
    }
}

#[function_component(SessionManagement)]
fn session_management(props: &SessionManagementProps) -> Html {
    let editing = use_state(|| None::<usize>);

    html! {
        <div class="autopeer-step">
            <ShellLine>
                <ShellPrompt>{"sessions"}</ShellPrompt>
                {" Manage your peering sessions"}
            </ShellLine>

            <div class="autopeer-sessions">
                if props.state.sessions.is_empty() {
                    <ShellLine>
                        <span class="text-secondary">{"No peering sessions configured yet."}</span>
                    </ShellLine>
                } else {
                    <div class="autopeer-sessions-list">
                        {for props.state.sessions.iter().enumerate().map(|(idx, session)| {
                            render_session_item(session, idx, &props.state, &editing)
                        })}
                    </div>
                }

                <div class="autopeer-new-session">
                    <ShellLine>
                        <ShellPrompt>{"new"}</ShellPrompt>
                        {" Create new peering session"}
                    </ShellLine>
                    <div class="autopeer-form">
                        <p class="text-secondary">{"Session creation form - to be implemented"}</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct SessionManagementProps {
    state: UseReducerHandle<AutoPeerState>,
}

fn render_session_management(state: &UseReducerHandle<AutoPeerState>) -> Html {
    html! {
        <SessionManagement state={state.clone()} />
    }
}

fn render_session_item(
    session: &PeeringSession,
    idx: usize,
    state: &UseReducerHandle<AutoPeerState>,
    editing: &UseStateHandle<Option<usize>>,
) -> Html {
    let is_editing = **editing == Some(idx);

    let editing_clone = editing.clone();
    let on_edit = Callback::from(move |_| {
        editing_clone.set(Some(idx));
    });

    let editing_clone = editing.clone();
    let on_cancel = Callback::from(move |_| {
        editing_clone.set(None);
    });

    let state_clone = state.clone();
    let on_delete = Callback::from(move |_| {
        // TODO: Send delete request to backend
        state_clone.dispatch(AutoPeerAction::SetLoading(true));
    });

    html! {
        <article class="peering-card peering-node">
            <div class="peering-node-header">
                <h4 class="peering-node-title">{&session.name}</h4>
                if let Some(comment) = &session.comment {
                    <span class="peering-node-meta">{comment}</span>
                }
            </div>

            if !is_editing {
                <>
                    <dl class="peering-grid">
                        if let Some(ipv4) = &session.ipv4 {
                            <>
                                <dt class="peering-label">{"IPv4"}</dt>
                                <dd class="peering-value">{ipv4}</dd>
                            </>
                        }
                        if let Some(ipv6) = &session.ipv6 {
                            <>
                                <dt class="peering-label">{"IPv6"}</dt>
                                <dd class="peering-value">{ipv6}</dd>
                            </>
                        }
                        <>
                            <dt class="peering-label">{"Endpoint"}</dt>
                            <dd class="peering-value">{&session.endpoint}</dd>
                        </>
                    </dl>

                    <div class="autopeer-session-actions">
                        <ShellButton text="Edit" onclick={on_edit} />
                        <ShellButton text="Delete" onclick={on_delete} />
                    </div>
                </>
            } else {
                <div class="autopeer-session-edit">
                    <p class="text-secondary">{"Edit form would go here"}</p>
                    <ShellButton text="Cancel" onclick={on_cancel} />
                </div>
            }
        </article>
    }
}
