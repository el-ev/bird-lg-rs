use common::models::PeeringInfo;
pub use dn42_auth_client::models::{AuthSessionResponse, UiMessage};
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SessionState {
    #[default]
    Managed,
    Manual,
    Locked,
    PendingPr,
    StalledPr,
    Conflict,
}

impl SessionState {
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Managed => "session_state.managed",
            Self::Manual => "session_state.manual",
            Self::Locked => "session_state.locked",
            Self::PendingPr => "session_state.pending_pr",
            Self::StalledPr => "session_state.stalled_pr",
            Self::Conflict => "session_state.conflict",
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
    pub const fn i18n_label_key(self) -> &'static str {
        match self {
            Self::FullTable => "peering_strategy.full_table.label",
            Self::Transit => "peering_strategy.transit.label",
            Self::Peer => "peering_strategy.peer.label",
            Self::Downstream => "peering_strategy.downstream.label",
        }
    }

    pub const fn i18n_description_key(self) -> &'static str {
        match self {
            Self::FullTable => "peering_strategy.full_table.description",
            Self::Transit => "peering_strategy.transit.description",
            Self::Peer => "peering_strategy.peer.description",
            Self::Downstream => "peering_strategy.downstream.description",
        }
    }

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
            PskField::Clear => serializer.serialize_str(""),
            PskField::Set(key) => serializer.serialize_str(key),
        }
    }
}

impl<'de> Deserialize<'de> for PskField {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(PskField::Unchanged),
            Some(key) if key.is_empty() => Ok(PskField::Clear),
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum OperationKind {
    #[default]
    Create,
    Update,
    Retire,
    Delete,
    Migrate,
}

impl OperationKind {
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Create => "operation.kind.create",
            Self::Update => "operation.kind.update",
            Self::Retire => "operation.kind.retire",
            Self::Delete => "operation.kind.delete",
            Self::Migrate => "operation.kind.migrate",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::PendingPullRequest => "operation.state.pending_pull_request",
            Self::PendingChecks => "operation.state.pending_checks",
            Self::Applying => "operation.state.applying",
            Self::PendingMerge => "operation.state.pending_merge",
            Self::Completed => "operation.state.completed",
            Self::Failed => "operation.state.failed",
            Self::Conflict => "operation.state.conflict",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Conflict)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationFailureStage {
    Checks,
    Preflight,
    Apply,
    Merge,
}

impl OperationFailureStage {
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Checks => "operation.failure_stage.checks",
            Self::Preflight => "operation.failure_stage.preflight",
            Self::Apply => "operation.failure_stage.apply",
            Self::Merge => "operation.failure_stage.merge",
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

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json, to_value};

    use super::*;

    #[test]
    fn psk_set_roundtrips() {
        let psk = PskField::Set("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into());
        let val = to_value(&psk).unwrap();
        assert_eq!(val, json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="));
        let back: PskField = from_value(val).unwrap();
        assert_eq!(back, psk);
    }

    #[test]
    fn psk_clear_serializes_as_empty_string() {
        let val = to_value(&PskField::Clear).unwrap();
        assert_eq!(val, json!(""));
    }

    #[test]
    fn psk_clear_roundtrips() {
        let val = to_value(&PskField::Clear).unwrap();
        let back: PskField = from_value(val).unwrap();
        assert_eq!(back, PskField::Clear);
    }

    #[test]
    fn psk_unchanged_serializes_as_null() {
        let val = to_value(&PskField::Unchanged).unwrap();
        assert_eq!(val, json!(null));
    }

    #[test]
    fn psk_null_deserializes_as_unchanged() {
        let back: PskField = from_value(json!(null)).unwrap();
        assert_eq!(back, PskField::Unchanged);
    }

    #[test]
    fn psk_unchanged_skipped_in_struct() {
        let spec = PeerSessionSpec {
            wg_public_key: "key".into(),
            ..Default::default()
        };
        let val = to_value(&spec).unwrap();
        assert!(!val.as_object().unwrap().contains_key("psk"));
    }

    #[test]
    fn psk_clear_present_in_struct() {
        let spec = PeerSessionSpec {
            wg_public_key: "key".into(),
            psk: PskField::Clear,
            ..Default::default()
        };
        let val = to_value(&spec).unwrap();
        assert_eq!(val["psk"], json!(""));
    }

    #[test]
    fn psk_absent_field_deserializes_as_unchanged() {
        let val = json!({
            "wg_public_key": "key",
            "ipv4": true,
            "ipv6": true,
            "extended_next_hop": true,
            "mp_bgp": true
        });
        let spec: PeerSessionSpec = from_value(val).unwrap();
        assert_eq!(spec.psk, PskField::Unchanged);
    }

    #[test]
    fn psk_empty_string_field_deserializes_as_clear() {
        let val = json!({
            "wg_public_key": "key",
            "ipv4": true,
            "ipv6": true,
            "extended_next_hop": true,
            "mp_bgp": true,
            "psk": ""
        });
        let spec: PeerSessionSpec = from_value(val).unwrap();
        assert_eq!(spec.psk, PskField::Clear);
    }
}
