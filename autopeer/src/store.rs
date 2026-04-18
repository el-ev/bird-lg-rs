use std::net::{Ipv4Addr, Ipv6Addr};

use common::auto_peer::{AuthSessionResponse, PeerSessionSpec, PeeringStrategy};
use serde::{Deserialize, Serialize};

const AUTOPEER_SESSION_STORAGE_KEY: &str = "bird-lg-rs.autopeer.sessions";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutoPeerStep {
    LoadingConfig,
    EnterAsn,
    SelectMethod,
    VerifyMethod,
    ManageSessions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerConfigStage {
    SelectNode,
    SessionDetails,
    Review,
}

impl PeerConfigStage {
    pub fn index(self) -> usize {
        match self {
            Self::SelectNode => 0,
            Self::SessionDetails => 1,
            Self::Review => 2,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::SelectNode => "Choose Node",
            Self::SessionDetails => "Configure Your Session",
            Self::Review => "Review Your Change",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::SelectNode => {
                "Choose the nearest node in our network before you fill in tunnel details."
            }
            Self::SessionDetails => {
                "Enter your WireGuard and BGP values, then adjust any advanced options you need."
            }
            Self::Review => "Review your change before we open the pull request.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionDraftField {
    Endpoint,
    WgPublicKey,
    Port,
    Peer4,
    Peer6,
    Own6,
    Keepalive,
    Mtu,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionDraft {
    pub node: String,
    pub comment: String,
    pub endpoint: String,
    pub wg_public_key: String,
    pub port: String,
    pub peer4: String,
    pub peer6: String,
    pub own6: String,
    pub keepalive: String,
    pub mtu: String,
    pub ipv4: bool,
    pub ipv6: bool,
    pub extended_next_hop: bool,
    pub mp_bgp: bool,
    pub peering_strategy: PeeringStrategy,
}

impl Default for SessionDraft {
    fn default() -> Self {
        Self {
            node: String::new(),
            comment: String::new(),
            endpoint: String::new(),
            wg_public_key: String::new(),
            port: String::new(),
            peer4: String::new(),
            peer6: String::new(),
            own6: String::new(),
            keepalive: String::new(),
            mtu: String::new(),
            ipv4: true,
            ipv6: true,
            extended_next_hop: true,
            mp_bgp: true,
            peering_strategy: PeeringStrategy::FullTable,
        }
    }
}

impl SessionDraft {
    pub fn default_for_asn(asn: &str) -> Self {
        Self {
            port: default_port_for_asn(asn),
            ..Self::default()
        }
    }

    pub fn from_session(node: &str, spec: &PeerSessionSpec) -> Self {
        Self {
            node: node.to_string(),
            comment: spec.comment.clone().unwrap_or_default(),
            endpoint: spec.endpoint.clone(),
            wg_public_key: spec.wg_public_key.clone(),
            port: spec.port.map(|value| value.to_string()).unwrap_or_default(),
            peer4: spec.peer4.clone().unwrap_or_default(),
            peer6: spec.peer6.clone().unwrap_or_default(),
            own6: spec.own6.clone().unwrap_or_default(),
            keepalive: spec
                .keepalive
                .map(|value| value.to_string())
                .unwrap_or_default(),
            mtu: spec.mtu.map(|value| value.to_string()).unwrap_or_default(),
            ipv4: spec.ipv4,
            ipv6: spec.ipv6,
            extended_next_hop: spec.extended_next_hop,
            mp_bgp: spec.mp_bgp,
            peering_strategy: spec.peering_strategy,
        }
    }

    pub fn resolved_port(&self, asn: &str) -> String {
        let trimmed = self.port.trim();
        if trimmed.is_empty() {
            default_port_for_asn(asn)
        } else {
            trimmed.to_string()
        }
    }

    pub fn families_label(&self) -> &'static str {
        match (self.ipv4, self.ipv6) {
            (true, true) => "IPv4 + IPv6",
            (true, false) => "IPv4 only",
            (false, true) => "IPv6 only",
            (false, false) => "No families selected",
        }
    }

    pub fn peer6_is_link_local(&self) -> bool {
        self.peer6.trim().to_lowercase().starts_with("fe80:")
    }

    pub fn field_error(&self, field: SessionDraftField) -> Option<String> {
        let peer4 = optional_string(&self.peer4);
        let peer6 = optional_string(&self.peer6);
        let own6 = optional_string(&self.own6);

        match field {
            SessionDraftField::Endpoint => validate_endpoint(&self.endpoint).err(),
            SessionDraftField::WgPublicKey => {
                validate_wireguard_public_key(&self.wg_public_key).err()
            }
            SessionDraftField::Port => optional_u16(&self.port, "WireGuard port").err(),
            SessionDraftField::Peer4 => {
                if !self.mp_bgp && self.ipv4 && peer4.is_none() {
                    Some(
                        "An IPv4 peer address is required for IPv4 when MP-BGP is disabled"
                            .to_string(),
                    )
                } else {
                    validate_peer_ipv4(peer4, "Peer IPv4 address").err()
                }
            }
            SessionDraftField::Peer6 => {
                if self.mp_bgp && peer6.is_none() {
                    Some("An IPv6 peer address is required when MP-BGP is enabled".to_string())
                } else if !self.mp_bgp && self.ipv6 && peer6.is_none() {
                    Some(
                        "An IPv6 peer address is required for IPv6 when MP-BGP is disabled"
                            .to_string(),
                    )
                } else {
                    validate_peer_ipv6(peer6, "Peer IPv6 address").err()
                }
            }
            SessionDraftField::Own6 => {
                if own6.is_some() && peer6.is_none() {
                    Some("A local link-local IPv6 needs a peer IPv6 address".to_string())
                } else if own6.is_some() && !self.peer6_is_link_local() {
                    Some("Local link-local IPv6 only applies when the peer IPv6 address is link-local".to_string())
                } else if let Some(local_ipv6) = own6.as_deref() {
                    if !local_ipv6.to_ascii_lowercase().starts_with("fe80:") {
                        Some("Local link-local IPv6 must start with fe80:".to_string())
                    } else {
                        validate_link_local_ipv6(own6, "Local link-local IPv6").err()
                    }
                } else {
                    None
                }
            }
            SessionDraftField::Keepalive => {
                optional_u16(&self.keepalive, "Persistent keepalive").err()
            }
            SessionDraftField::Mtu => validate_optional_mtu(&self.mtu).err(),
        }
    }

    pub fn to_spec(&self) -> Result<PeerSessionSpec, String> {
        let peer4 = optional_string(&self.peer4);
        let peer6 = optional_string(&self.peer6);
        let own6 = optional_string(&self.own6);

        if peer4.is_none() && peer6.is_none() {
            return Err("Add at least one tunnel address: IPv4 or IPv6".to_string());
        }
        if !self.ipv4 && !self.ipv6 {
            return Err("Enable at least one BGP family".to_string());
        }
        if self.mp_bgp && peer6.is_none() {
            return Err("An IPv6 peer address is required when MP-BGP is enabled".to_string());
        }
        if !self.mp_bgp && self.ipv4 && peer4.is_none() {
            return Err(
                "An IPv4 peer address is required for IPv4 when MP-BGP is disabled".to_string(),
            );
        }
        if !self.mp_bgp && self.ipv6 && peer6.is_none() {
            return Err(
                "An IPv6 peer address is required for IPv6 when MP-BGP is disabled".to_string(),
            );
        }
        if self.extended_next_hop && !self.mp_bgp {
            return Err("Extended next hop requires MP-BGP".to_string());
        }
        if own6.is_some() && peer6.is_none() {
            return Err("A local link-local IPv6 needs a peer IPv6 address".to_string());
        }
        if own6.is_some() && !self.peer6_is_link_local() {
            return Err(
                "Local link-local IPv6 only applies when the peer IPv6 address is link-local"
                    .to_string(),
            );
        }

        let endpoint = validate_endpoint(&self.endpoint)?;
        let wg_public_key = validate_wireguard_public_key(&self.wg_public_key)?;
        let peer4 = validate_peer_ipv4(peer4, "Peer IPv4 address")?;
        let peer6 = validate_peer_ipv6(peer6, "Peer IPv6 address")?;

        if let Some(local_ipv6) = own6.as_deref() {
            if !local_ipv6.to_ascii_lowercase().starts_with("fe80:") {
                return Err("Local link-local IPv6 must start with fe80:".to_string());
            }
        }
        let own6 = validate_link_local_ipv6(own6, "Local link-local IPv6")?;
        let mtu = validate_optional_mtu(&self.mtu)?;

        Ok(PeerSessionSpec {
            comment: (!self.comment.trim().is_empty()).then(|| self.comment.trim().to_string()),
            endpoint,
            wg_public_key,
            port: optional_u16(&self.port, "port")?,
            peer4,
            peer6,
            own6,
            keepalive: optional_u16(&self.keepalive, "keepalive")?,
            mtu,
            ipv4: self.ipv4,
            ipv6: self.ipv6,
            extended_next_hop: self.extended_next_hop,
            mp_bgp: self.mp_bgp,
            peering_strategy: self.peering_strategy,
        })
    }
}

fn default_port_for_asn(asn: &str) -> String {
    asn.chars()
        .rev()
        .take(5)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedSessions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_session: Option<AuthSessionResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_session: Option<AuthSessionResponse>,
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn load_persisted_sessions() -> Option<PersistedSessions> {
    let raw = local_storage()?
        .get_item(AUTOPEER_SESSION_STORAGE_KEY)
        .ok()
        .flatten()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_persisted_sessions(sessions: &PersistedSessions) {
    let Some(storage) = local_storage() else {
        return;
    };

    if sessions.auth_session.is_none() && sessions.host_session.is_none() {
        let _ = storage.remove_item(AUTOPEER_SESSION_STORAGE_KEY);
        return;
    }

    if let Ok(value) = serde_json::to_string(sessions) {
        let _ = storage.set_item(AUTOPEER_SESSION_STORAGE_KEY, &value);
    }
}

fn required(value: &str, field: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("{field} is required"))
    } else {
        Ok(trimmed.to_string())
    }
}

fn validate_endpoint(value: &str) -> Result<String, String> {
    let endpoint = required(value, "endpoint")?;

    if endpoint.contains(char::is_whitespace) {
        return Err("Remote endpoint cannot contain spaces".to_string());
    }

    if let Some(host) = endpoint.strip_prefix('[') {
        let Some((ipv6, port)) = host.split_once("]:") else {
            return Err("IPv6 endpoints must use the format [addr]:port".to_string());
        };
        parse_ipv6(ipv6, "Remote endpoint IPv6 address")?;
        parse_port_component(port)?;
        return Ok(endpoint);
    }

    if endpoint.matches(':').count() != 1 {
        return Err("Remote endpoint must use host:port or [ipv6]:port".to_string());
    }

    let Some((host, port)) = endpoint.rsplit_once(':') else {
        return Err("Remote endpoint must include a port".to_string());
    };
    if host.is_empty() {
        return Err("Remote endpoint host is required".to_string());
    }
    if host.parse::<Ipv4Addr>().is_err() && !is_valid_endpoint_hostname(host) {
        return Err(
            "Remote endpoint host must be an IPv4 address or a fully qualified hostname"
                .to_string(),
        );
    }
    parse_port_component(port)?;
    Ok(endpoint)
}

fn validate_wireguard_public_key(value: &str) -> Result<String, String> {
    let key = required(value, "wg_public_key")?;

    if key.len() != 44 {
        return Err("Peer WireGuard key must be a 44-character base64 public key".to_string());
    }
    if !key.ends_with('=') {
        return Err("Peer WireGuard key must end with '='".to_string());
    }
    if !key
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '+' | '/' | '='))
    {
        return Err("Peer WireGuard key contains invalid base64 characters".to_string());
    }

    Ok(key)
}

fn validate_peer_ipv4(value: Option<String>, label: &str) -> Result<Option<String>, String> {
    match value {
        Some(value) => {
            let addr = parse_ipv4(&value, label)?;
            if !is_dn42_tunnel_ipv4(addr) {
                // TODO: too tight?
                return Err(format!("{label} must be within 172.20.0.0/14"));
            }
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn validate_peer_ipv6(value: Option<String>, label: &str) -> Result<Option<String>, String> {
    match value {
        Some(value) => {
            let addr = parse_ipv6(&value, label)?;
            if !is_link_local_ipv6(addr) && !is_unique_local_ipv6(addr) {
                return Err(format!("{label} must be a ULA or link-local IPv6 address"));
            }
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn validate_link_local_ipv6(value: Option<String>, label: &str) -> Result<Option<String>, String> {
    match value {
        Some(value) => {
            let addr = parse_ipv6(&value, label)?;
            if !is_link_local_ipv6(addr) {
                return Err(format!("{label} must be a link-local IPv6 address"));
            }
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn validate_optional_mtu(value: &str) -> Result<Option<u16>, String> {
    let mtu = optional_u16(value, "Interface MTU")?;
    if let Some(mtu) = mtu {
        if !(1280..=1500).contains(&mtu) {
            return Err("Interface MTU must be between 1280 and 1500".to_string());
        }
    }
    Ok(mtu)
}

fn parse_ipv4(value: &str, label: &str) -> Result<Ipv4Addr, String> {
    value
        .parse::<Ipv4Addr>()
        .map_err(|_| format!("{label} must be a valid IPv4 address"))
}

fn parse_ipv6(value: &str, label: &str) -> Result<Ipv6Addr, String> {
    value
        .parse::<Ipv6Addr>()
        .map_err(|_| format!("{label} must be a valid IPv6 address"))
}

fn parse_port_component(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| "Remote endpoint port must be a valid number".to_string())
        .and_then(|port| {
            if port == 0 {
                Err("Remote endpoint port must be between 1 and 65535".to_string())
            } else {
                Ok(port)
            }
        })
}

fn is_valid_hostname(value: &str) -> bool {
    value.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|char| char.is_ascii_alphanumeric() || char == '-')
    })
}

fn is_valid_endpoint_hostname(value: &str) -> bool {
    let trimmed = value.trim_end_matches('.');
    trimmed.contains('.')
        && trimmed.chars().any(|char| char.is_ascii_alphabetic())
        && is_valid_hostname(trimmed)
}

fn is_dn42_tunnel_ipv4(value: Ipv4Addr) -> bool {
    let [first, second, ..] = value.octets();
    first == 172 && (20..=23).contains(&second)
}

fn is_link_local_ipv6(value: Ipv6Addr) -> bool {
    value.segments()[0] == 0xfe80
}

fn is_unique_local_ipv6(value: Ipv6Addr) -> bool {
    (value.segments()[0] & 0xfe00) == 0xfc00
}

fn optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn optional_u16(value: &str, field: &str) -> Result<Option<u16>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    trimmed
        .parse::<u16>()
        .map(Some)
        .map_err(|_| format!("{field} must be a valid number"))
}

#[cfg(test)]
mod tests {
    use common::auto_peer::{AuthMethod, AuthMethodKind, AuthSessionResponse, PeeringStrategy};

    use super::{PersistedSessions, SessionDraft};

    const VALID_WG_KEY: &str = "sLbzTRr2gfLFb24NPzDOpy8j09Y6zI+a7NkeVMdVSR8=";

    #[test]
    fn default_port_uses_last_five_asn_digits() {
        let draft = SessionDraft::default_for_asn("4242423914");
        assert_eq!(draft.port, "23914");
    }

    #[test]
    fn resolved_port_falls_back_to_last_five_digits() {
        let mut draft = SessionDraft::default_for_asn("4242423914");
        draft.port.clear();
        assert_eq!(draft.resolved_port("4242423914"), "23914");
    }

    #[test]
    fn draft_to_spec_trims_optional_fields() {
        let draft = SessionDraft {
            endpoint: " peer.example.net:21023 ".into(),
            wg_public_key: format!(" {VALID_WG_KEY} "),
            peer6: " fe80::1234 ".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.endpoint, "peer.example.net:21023");
        assert_eq!(spec.wg_public_key, VALID_WG_KEY);
        assert_eq!(spec.peer6, Some("fe80::1234".into()));
        assert_eq!(spec.port, Some(21234));
        assert_eq!(spec.peering_strategy, PeeringStrategy::FullTable);
    }

    #[test]
    fn draft_to_spec_preserves_non_default_peering_strategy() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            peering_strategy: PeeringStrategy::Downstream,
            ..SessionDraft::default_for_asn("4242421234")
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.peering_strategy, PeeringStrategy::Downstream);
    }

    #[test]
    fn draft_to_spec_allows_ipv6_ula() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            peer6: "fd55:dead:beef::3".into(),
            ..SessionDraft::default_for_asn("4242422172")
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.peer4, Some("172.20.193.67".into()));
        assert_eq!(spec.peer6, Some("fd55:dead:beef::3".into()));
    }

    #[test]
    fn draft_to_spec_rejects_invalid_wireguard_key() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "not-a-key".into(),
            peer6: "fe80::1234".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Peer WireGuard key must be a 44-character base64 public key"
        );
    }

    #[test]
    fn draft_to_spec_rejects_invalid_ipv4_address() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.999.67".into(),
            peer6: "fd55:dead:beef::3".into(),
            ..SessionDraft::default_for_asn("4242422172")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Peer IPv4 address must be a valid IPv4 address"
        );
    }

    #[test]
    fn draft_to_spec_rejects_invalid_endpoint_shape() {
        let draft = SessionDraft {
            endpoint: "peer.example.net".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Remote endpoint must use host:port or [ipv6]:port"
        );
    }

    #[test]
    fn draft_to_spec_rejects_non_fqdn_endpoint_host() {
        let draft = SessionDraft {
            endpoint: "1:2".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Remote endpoint host must be an IPv4 address or a fully qualified hostname"
        );
    }

    #[test]
    fn draft_to_spec_rejects_peer4_outside_dn42_range() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "0.0.0.0".into(),
            peer6: "fd55:dead:beef::3".into(),
            ..SessionDraft::default_for_asn("4242422172")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Peer IPv4 address must be within 172.20.0.0/14"
        );
    }

    #[test]
    fn draft_to_spec_rejects_peer6_outside_ula_and_link_local() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "::".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Peer IPv6 address must be a ULA or link-local IPv6 address"
        );
    }

    #[test]
    fn draft_to_spec_rejects_out_of_range_mtu() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            mtu: "11451".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Interface MTU must be between 1280 and 1500"
        );
    }

    #[test]
    fn draft_to_spec_rejects_non_link_local_own6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            own6: "fd42::1".into(),
            ..SessionDraft::default_for_asn("4242421234")
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "Local link-local IPv6 must start with fe80:"
        );
    }

    #[test]
    fn persisted_sessions_roundtrip() {
        let sessions = PersistedSessions {
            auth_session: Some(AuthSessionResponse {
                session_token: "token".into(),
                asn: "4242421234".into(),
                effective_mnt: "EXAMPLE-MNT".into(),
                auth_method: AuthMethod {
                    kind: AuthMethodKind::RegistrySsh,
                    label: "Registry SSH Signature".into(),
                    description: "Signed with maintainer SSH key".into(),
                    provider: None,
                    ssh_fingerprints: Vec::new(),
                    pgp_fingerprints: Vec::new(),
                    email_targets: Vec::new(),
                },
                can_impersonate: true,
                expires_at: "2026-04-18T10:00:00Z".into(),
            }),
            host_session: None,
        };

        let encoded = serde_json::to_string(&sessions).unwrap();
        let decoded: PersistedSessions = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, sessions);
    }
}
