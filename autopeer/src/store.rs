#![allow(dead_code)]

use std::rc::Rc;

use common::auto_peer::{AutoPeerResponse, ChallengeMethod, PeeringSession};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AutoPeerStep {
    EnterAsn,
    SelectChallenge,
    VerifyPgp,
    VerifyEmail,
    ManageSessions,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AutoPeerState {
    pub(crate) step: AutoPeerStep,
    pub(crate) asn: String,
    pub(crate) challenge_methods: Vec<ChallengeMethod>,
    pub(crate) selected_method: Option<ChallengeMethod>,
    pub(crate) challenge_text: Option<String>,
    pub(crate) credential: Option<String>,
    pub(crate) sessions: Vec<PeeringSession>,
    pub(crate) error: Option<String>,
    pub(crate) loading: bool,
}

impl Default for AutoPeerState {
    fn default() -> Self {
        Self {
            step: AutoPeerStep::EnterAsn,
            asn: String::new(),
            challenge_methods: Vec::new(),
            selected_method: None,
            challenge_text: None,
            credential: None,
            sessions: Vec::new(),
            error: None,
            loading: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AutoPeerAction {
    SetAsn(String),
    SetLoading(bool),
    SetError(Option<String>),
    HandleInitResponse(AutoPeerResponse),
    SelectMethod(ChallengeMethod),
    HandleChallengeResponse(AutoPeerResponse),
    SetPgpPubkey(String),
    SetPgpSignature(String),
    SetEmailCode(String),
    HandleVerifyResponse(AutoPeerResponse),
    HandleSessionsUpdate(Vec<PeeringSession>),
    Reset,
}

impl Reducible for AutoPeerState {
    type Action = AutoPeerAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();

        match action {
            AutoPeerAction::SetAsn(asn) => {
                state.asn = asn;
            }
            AutoPeerAction::SetLoading(loading) => {
                state.loading = loading;
            }
            AutoPeerAction::SetError(error) => {
                state.error = error;
                state.loading = false;
            }
            AutoPeerAction::HandleInitResponse(response) => {
                state.loading = false;
                match response {
                    AutoPeerResponse::InitSuccess { challenge_methods } => {
                        state.challenge_methods = challenge_methods;
                        state.step = AutoPeerStep::SelectChallenge;
                        state.error = None;
                    }
                    AutoPeerResponse::InitError { error } => {
                        state.error = Some(error);
                    }
                    _ => {}
                }
            }
            AutoPeerAction::SelectMethod(method) => {
                state.selected_method = Some(method);
            }
            AutoPeerAction::HandleChallengeResponse(response) => {
                state.loading = false;
                match response {
                    AutoPeerResponse::ChallengeSelected { challenge_text } => {
                        state.challenge_text = challenge_text;
                        state.step = if state.selected_method == Some(ChallengeMethod::Pgp) {
                            AutoPeerStep::VerifyPgp
                        } else {
                            AutoPeerStep::VerifyEmail
                        };
                        state.error = None;
                    }
                    AutoPeerResponse::OperationError { error } => {
                        state.error = Some(error);
                    }
                    _ => {}
                }
            }
            AutoPeerAction::HandleVerifyResponse(response) => {
                state.loading = false;
                match response {
                    AutoPeerResponse::VerifySuccess {
                        credential,
                        sessions,
                    } => {
                        state.credential = Some(credential);
                        state.sessions = sessions;
                        state.step = AutoPeerStep::ManageSessions;
                        state.error = None;
                    }
                    AutoPeerResponse::VerifyError { error } => {
                        state.error = Some(error);
                    }
                    _ => {}
                }
            }
            AutoPeerAction::HandleSessionsUpdate(sessions) => {
                state.sessions = sessions;
                state.loading = false;
            }
            AutoPeerAction::Reset => {
                state = Self::default();
            }
            _ => {}
        }

        Rc::new(state)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use common::auto_peer::{AutoPeerResponse, ChallengeMethod, PeeringSession};
    use yew::Reducible;

    use super::{AutoPeerAction, AutoPeerState, AutoPeerStep};

    fn reduce(state: AutoPeerState, action: AutoPeerAction) -> AutoPeerState {
        Rc::new(state).reduce(action).as_ref().clone()
    }

    #[test]
    fn init_success_advances_to_challenge_selection() {
        let state = reduce(
            AutoPeerState {
                loading: true,
                ..AutoPeerState::default()
            },
            AutoPeerAction::HandleInitResponse(AutoPeerResponse::InitSuccess {
                challenge_methods: vec![ChallengeMethod::Pgp, ChallengeMethod::Email],
            }),
        );

        assert_eq!(state.step, AutoPeerStep::SelectChallenge);
        assert_eq!(
            state.challenge_methods,
            vec![ChallengeMethod::Pgp, ChallengeMethod::Email]
        );
        assert!(!state.loading);
        assert_eq!(state.error, None);
    }

    #[test]
    fn challenge_selection_advances_to_matching_verification_step() {
        let state = reduce(
            reduce(
                AutoPeerState::default(),
                AutoPeerAction::SelectMethod(ChallengeMethod::Pgp),
            ),
            AutoPeerAction::HandleChallengeResponse(AutoPeerResponse::ChallengeSelected {
                challenge_text: Some("challenge".to_string()),
            }),
        );

        assert_eq!(state.step, AutoPeerStep::VerifyPgp);
        assert_eq!(state.challenge_text, Some("challenge".to_string()));
        assert_eq!(state.error, None);
    }

    #[test]
    fn verify_success_stores_credential_and_sessions() {
        let session = PeeringSession {
            id: Some("session-1".to_string()),
            name: "Example Peer".to_string(),
            ipv4: Some("172.20.0.1".to_string()),
            ipv6: Some("fd00::1".to_string()),
            endpoint: "peer.example.net:51820".to_string(),
            comment: Some("Primary peering".to_string()),
        };

        let state = reduce(
            AutoPeerState {
                loading: true,
                ..AutoPeerState::default()
            },
            AutoPeerAction::HandleVerifyResponse(AutoPeerResponse::VerifySuccess {
                credential: "credential-token".to_string(),
                sessions: vec![session.clone()],
            }),
        );

        assert_eq!(state.step, AutoPeerStep::ManageSessions);
        assert_eq!(state.credential, Some("credential-token".to_string()));
        assert_eq!(state.sessions, vec![session]);
        assert!(!state.loading);
        assert_eq!(state.error, None);
    }

    #[test]
    fn init_error_clears_loading_and_sets_error() {
        let state = reduce(
            AutoPeerState {
                loading: true,
                ..AutoPeerState::default()
            },
            AutoPeerAction::HandleInitResponse(AutoPeerResponse::InitError {
                error: "ASN not found".to_string(),
            }),
        );

        assert!(!state.loading);
        assert_eq!(state.error, Some("ASN not found".to_string()));
        assert_eq!(state.step, AutoPeerStep::EnterAsn);
    }

    #[test]
    fn verify_error_clears_loading_and_sets_error() {
        let state = reduce(
            AutoPeerState {
                loading: true,
                ..AutoPeerState::default()
            },
            AutoPeerAction::HandleVerifyResponse(AutoPeerResponse::VerifyError {
                error: "Verification failed".to_string(),
            }),
        );

        assert!(!state.loading);
        assert_eq!(state.error, Some("Verification failed".to_string()));
        assert_eq!(state.step, AutoPeerStep::EnterAsn);
    }
}
