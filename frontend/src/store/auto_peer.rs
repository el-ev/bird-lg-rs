#![allow(dead_code)]
use std::rc::Rc;

use common::auto_peer::{AutoPeerResponse, ChallengeMethod, PeeringSession};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AutoPeerStep {
    EnterAsn,
    SelectChallenge,
    VerifyPgp,
    VerifyEmail,
    ManageSessions,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutoPeerState {
    pub step: AutoPeerStep,
    pub asn: String,
    pub challenge_methods: Vec<ChallengeMethod>,
    pub selected_method: Option<ChallengeMethod>,
    pub challenge_text: Option<String>,
    pub credential: Option<String>,
    pub sessions: Vec<PeeringSession>,
    pub error: Option<String>,
    pub loading: bool,
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
pub enum AutoPeerAction {
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
                        state.challenge_text = challenge_text.clone();
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

pub type AutoPeerStateHandle = UseReducerHandle<AutoPeerState>;
