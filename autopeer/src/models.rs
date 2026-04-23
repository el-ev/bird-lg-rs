use std::collections::BTreeMap;

use common::models::PeeringInfo;
use serde::de::Deserializer;
use serde::ser::Serializer;
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SessionState {
    #[default]
    Managed,
    Manual,
    PendingPr,
    StalledPr,
    Conflict,
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
pub enum MpBgpTransport {
    Ipv4,
    #[default]
    Ipv6,
}

impl MpBgpTransport {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "ipv4" => Some(Self::Ipv4),
            "ipv6" => Some(Self::Ipv6),
            _ => None,
        }
    }
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

pub const ALL_MP_BGP_TRANSPORTS: [MpBgpTransport; 2] = [MpBgpTransport::Ipv4, MpBgpTransport::Ipv6];

fn is_default_peering_strategy(strategy: &PeeringStrategy) -> bool {
    matches!(strategy, PeeringStrategy::FullTable)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PskField {
    #[default]
    Unchanged,
    Clear,
    Set(String),
}

impl PskField {
    fn is_unchanged(&self) -> bool {
        matches!(self, PskField::Unchanged)
    }
}

impl Serialize for PskField {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            PskField::Unchanged => serializer.serialize_none(),
            PskField::Clear => serializer.serialize_none(),
            PskField::Set(key) => serializer.serialize_str(key),
        }
    }
}

impl<'de> Deserialize<'de> for PskField {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(PskField::Clear),
            Some(key) => Ok(PskField::Set(key)),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerSessionSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mp_bgp_transport: Option<MpBgpTransport>,
    #[serde(default, skip_serializing_if = "is_default_peering_strategy")]
    pub peering_strategy: PeeringStrategy,
    #[serde(default, skip_serializing_if = "PskField::is_unchanged")]
    pub psk: PskField,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypt_endpoint: Option<bool>,
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
    #[serde(default)]
    pub has_psk: bool,
    #[serde(default)]
    pub has_encrypted_endpoint: bool,
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
    #[serde(rename = "delete")]
    Retire,
    #[serde(rename = "purge")]
    Delete,
    Migrate,
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
