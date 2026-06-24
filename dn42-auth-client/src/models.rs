use std::collections::BTreeMap;

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
pub struct PgpKeyLookupResponse {
    pub fingerprint: String,
    #[serde(default)]
    pub found: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEmailSendRequest {
    pub challenge_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_mnt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_to: Option<String>,
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
