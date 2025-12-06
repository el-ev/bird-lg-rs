use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t")]
pub enum AutoPeerRequest {
    #[serde(rename = "init")]
    InitSession { asn: String },
    #[serde(rename = "challenge")]
    SelectChallenge { method: ChallengeMethod },
    #[serde(rename = "verify_pgp")]
    VerifyPgp { pubkey: String, signature: String },
    #[serde(rename = "verify_email")]
    VerifyEmail { code: String },
    #[serde(rename = "get_sessions")]
    GetSessions,
    #[serde(rename = "create_session")]
    CreateSession { session: PeeringSession },
    #[serde(rename = "update_session")]
    UpdateSession { id: String, session: PeeringSession },
    #[serde(rename = "delete_session")]
    DeleteSession { id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t")]
pub enum AutoPeerResponse {
    #[serde(rename = "init_success")]
    InitSuccess {
        challenge_methods: Vec<ChallengeMethod>,
    },
    #[serde(rename = "init_error")]
    InitError { error: String },
    #[serde(rename = "challenge_selected")]
    ChallengeSelected {
        #[serde(skip_serializing_if = "Option::is_none")]
        challenge_text: Option<String>,
    },
    #[serde(rename = "verify_success")]
    VerifySuccess {
        credential: String,
        sessions: Vec<PeeringSession>,
    },
    #[serde(rename = "verify_error")]
    VerifyError { error: String },
    #[serde(rename = "sessions_update")]
    SessionsUpdate { sessions: Vec<PeeringSession> },
    #[serde(rename = "operation_success")]
    OperationSuccess { message: String },
    #[serde(rename = "operation_error")]
    OperationError { error: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeMethod {
    Pgp,
    Email,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PeeringSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
