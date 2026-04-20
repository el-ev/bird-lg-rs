use std::collections::BTreeMap;

use common::models::PeeringInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiMessage {
    pub key: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
}

impl UiMessage {
    pub fn key(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ..Self::default()
        }
    }

    pub fn raw(value: impl Into<String>) -> Self {
        Self {
            key: value.into(),
            params: BTreeMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn is_key(&self, expected: &str) -> bool {
        self.key == expected
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AuthMethodKind {
    #[default]
    RegistrySsh,
    RegistryPgp,
    RegistryEmail,
    Oidc,
    HostImpersonation,
}

impl AuthMethodKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::RegistrySsh => "Registry SSH Signature",
            Self::RegistryPgp => "Registry PGP Signature",
            Self::RegistryEmail => "Registry Email",
            Self::Oidc => "Third-Party Login",
            Self::HostImpersonation => "Host ASN Impersonation",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEmailTarget {
    pub maintainer: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthMethod {
    pub kind: AuthMethodKind,
    pub label: UiMessage,
    pub description: UiMessage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ssh_fingerprints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pgp_fingerprints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub email_targets: Vec<RegistryEmailTarget>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthStartRequest {
    pub asn: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthStartResponse {
    pub asn: String,
    pub challenge_id: String,
    pub challenge_text: String,
    pub challenge_ttl_seconds: u64,
    #[serde(default)]
    pub methods: Vec<AuthMethod>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistrySshVerifyRequest {
    pub challenge_id: String,
    pub signature: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryPgpVerifyRequest {
    pub challenge_id: String,
    pub public_key: String,
    pub signed_message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEmailSendRequest {
    pub challenge_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_mnt: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEmailSendResponse {
    pub effective_mnt: String,
    #[serde(default)]
    pub emails: Vec<String>,
    pub expires_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEmailVerifyRequest {
    pub challenge_id: String,
    pub code: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEmailCompleteRequest {
    pub token: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OidcStartRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenge_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OidcStartResponse {
    pub authorization_url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OidcCompleteRequest {
    pub state: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostImpersonationRequest {
    pub asn: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_mnt: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthSessionResponse {
    pub session_token: String,
    pub asn: String,
    pub effective_mnt: String,
    pub auth_method: AuthMethod,
    #[serde(default)]
    pub can_impersonate: bool,
    pub expires_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeView {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    pub ip_support: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peering: Option<PeeringInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autopeer: Option<bool>,
}

impl NodeView {
    pub fn summary(&self) -> String {
        let mut parts = vec![self.name.clone()];

        if let Some(region) = &self.region {
            parts.push(region.clone());
        }
        if let Some(country) = &self.country {
            parts.push(country.clone());
        }
        parts.push(self.ip_support.clone());

        parts.join(" / ")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SessionState {
    #[default]
    Managed,
    Manual,
    PendingPr,
    Conflict,
}

impl SessionState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Managed => "Managed",
            Self::Manual => "Manual",
            Self::PendingPr => "Pending PR",
            Self::Conflict => "Conflict",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMetadata {
    pub managed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_mnt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_provider: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PeeringStrategy {
    #[default]
    FullTable,
    Transit,
    Peer,
    Downstream,
}

impl PeeringStrategy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullTable => "full_table",
            Self::Transit => "transit",
            Self::Peer => "peer",
            Self::Downstream => "downstream",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::FullTable => "Full Table",
            Self::Transit => "Transit",
            Self::Peer => "Peer",
            Self::Downstream => "Downstream",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::FullTable => "Receive all valid routes and export all valid routes.",
            Self::Transit => "Receive all valid routes and export only our own exact prefixes.",
            Self::Peer => {
                "Receive only direct routes and export our own exact prefixes plus downstream routes."
            }
            Self::Downstream => "Receive only direct routes and export all valid routes.",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "full_table" => Some(Self::FullTable),
            "transit" => Some(Self::Transit),
            "peer" => Some(Self::Peer),
            "downstream" => Some(Self::Downstream),
            _ => None,
        }
    }
}

pub const ALL_PEERING_STRATEGIES: [PeeringStrategy; 4] = [
    PeeringStrategy::FullTable,
    PeeringStrategy::Transit,
    PeeringStrategy::Peer,
    PeeringStrategy::Downstream,
];

fn is_default_peering_strategy(strategy: &PeeringStrategy) -> bool {
    matches!(strategy, PeeringStrategy::FullTable)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerSessionSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub endpoint: String,
    pub wg_public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer4: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer6: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub own6: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keepalive: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u16>,
    #[serde(default = "default_true")]
    pub ipv4: bool,
    #[serde(default = "default_true")]
    pub ipv6: bool,
    #[serde(default = "default_true")]
    pub extended_next_hop: bool,
    #[serde(default = "default_true")]
    pub mp_bgp: bool,
    #[serde(default, skip_serializing_if = "is_default_peering_strategy")]
    pub peering_strategy: PeeringStrategy,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionView {
    pub node: String,
    pub asn: String,
    pub state: SessionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<PeerSessionSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SessionMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<UiMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionListResponse {
    pub asn: String,
    #[serde(default)]
    pub nodes: Vec<NodeView>,
    #[serde(default)]
    pub sessions: Vec<SessionView>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateSessionRequest {
    pub node: String,
    pub session: PeerSessionSpec,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateSessionRequest {
    pub session: PeerSessionSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum OperationKind {
    #[default]
    Create,
    Update,
    Delete,
    Migrate,
}

impl OperationKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::Update => "Update",
            Self::Delete => "Delete",
            Self::Migrate => "Migrate",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum OperationState {
    #[default]
    PendingPullRequest,
    PendingChecks,
    Applying,
    PendingMerge,
    Completed,
    Failed,
    Conflict,
}

impl OperationState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PendingPullRequest => "Preparing PR",
            Self::PendingChecks => "Waiting For CI",
            Self::Applying => "Applying On Node",
            Self::PendingMerge => "Waiting For Merge",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Conflict => "Conflict",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Conflict)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationFailureStage {
    Checks,
    Preflight,
    Apply,
    Merge,
}

impl OperationFailureStage {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Checks => "CI checks",
            Self::Preflight => "Node preflight",
            Self::Apply => "Node apply",
            Self::Merge => "Merge",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationFailureDetails {
    pub stage: OperationFailureStage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationStatus {
    pub id: String,
    pub asn: String,
    pub node: String,
    pub kind: OperationKind,
    pub state: OperationState,
    pub branch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_run_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<UiMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_details: Option<OperationFailureDetails>,
    pub created_at: String,
    pub updated_at: String,
}
