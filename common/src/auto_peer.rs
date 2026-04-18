use serde::{Deserialize, Serialize};

use crate::models::PeeringInfo;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethodKind {
    RegistrySsh,
    RegistryPgp,
    Oidc,
    HostImpersonation,
}

impl Default for AuthMethodKind {
    fn default() -> Self {
        Self::RegistrySsh
    }
}

impl AuthMethodKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::RegistrySsh => "Registry SSH Signature",
            Self::RegistryPgp => "Registry PGP Signature",
            Self::Oidc => "Third-Party Login",
            Self::HostImpersonation => "Host ASN Impersonation",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthMethod {
    pub kind: AuthMethodKind,
    pub label: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ssh_fingerprints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pgp_fingerprints: Vec<String>,
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
pub enum SessionState {
    Managed,
    Manual,
    PendingPr,
    Conflict,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Managed
    }
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
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionListResponse {
    pub asn: String,
    pub effective_mnt: String,
    pub auth_method: AuthMethod,
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
pub enum OperationKind {
    Create,
    Update,
    Delete,
    Migrate,
}

impl Default for OperationKind {
    fn default() -> Self {
        Self::Create
    }
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
pub enum OperationState {
    PendingPullRequest,
    PendingChecks,
    PendingMerge,
    Merged,
    Applying,
    Completed,
    Failed,
    Conflict,
}

impl Default for OperationState {
    fn default() -> Self {
        Self::PendingPullRequest
    }
}

impl OperationState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PendingPullRequest => "Preparing PR",
            Self::PendingChecks => "Waiting For CI",
            Self::PendingMerge => "Waiting For Merge",
            Self::Merged => "Merged",
            Self::Applying => "Applying",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Conflict => "Conflict",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Conflict)
    }
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
    pub message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
