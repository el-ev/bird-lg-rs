use std::net::{Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

use crate::models::{AuthSessionResponse, MpBgpTransport, PeerSessionSpec, PeeringStrategy};

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

    pub fn title_key(self) -> &'static str {
        match self {
            Self::SelectNode => "flow.select_node.title",
            Self::SessionDetails => "flow.session_details.title",
            Self::Review => "flow.review.title",
        }
    }

    pub fn description_key(self) -> &'static str {
        match self {
            Self::SelectNode => "flow.select_node.description",
            Self::SessionDetails => "flow.session_details.description",
            Self::Review => "flow.review.description",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionDraftField {
    Endpoint,
    WgPublicKey,
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
    pub peer4: String,
    pub peer6: String,
    pub own6: String,
    pub keepalive: String,
    pub mtu: String,
    pub ipv4: bool,
    pub ipv6: bool,
    pub extended_next_hop: bool,
    pub mp_bgp: bool,
    pub mp_bgp_transport: Option<MpBgpTransport>,
    pub peering_strategy: PeeringStrategy,
    pub psk: String,
    pub encrypt_endpoint: bool,
}

impl Default for SessionDraft {
    fn default() -> Self {
        Self {
            node: String::new(),
            comment: String::new(),
            endpoint: String::new(),
            wg_public_key: String::new(),
            peer4: String::new(),
            peer6: String::new(),
            own6: String::new(),
            keepalive: String::new(),
            mtu: String::new(),
            ipv4: true,
            ipv6: true,
            extended_next_hop: true,
            mp_bgp: true,
            mp_bgp_transport: None,
            peering_strategy: PeeringStrategy::FullTable,
            psk: String::new(),
            encrypt_endpoint: false,
        }
    }
}

impl SessionDraft {
    pub fn from_session(node: &str, spec: &PeerSessionSpec) -> Self {
        Self {
            node: node.to_string(),
            comment: spec.comment.clone().unwrap_or_default(),
            endpoint: spec.endpoint.clone(),
            wg_public_key: spec.wg_public_key.clone(),
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
            mp_bgp_transport: spec.mp_bgp_transport,
            peering_strategy: spec.peering_strategy,
            psk: String::new(),
            encrypt_endpoint: spec.encrypt_endpoint.unwrap_or(false),
        }
    }

    pub fn from_session_view(node: &str, view: &crate::models::SessionView) -> Self {
        let mut draft = view
            .spec
            .as_ref()
            .map(|spec| Self::from_session(node, spec))
            .unwrap_or_else(|| Self {
                node: node.to_string(),
                ..Default::default()
            });
        draft.encrypt_endpoint = view.has_encrypted_endpoint;
        draft
    }

    pub fn families_label_key(&self) -> &'static str {
        match (self.ipv4, self.ipv6) {
            (true, true) => "draft.families.ipv4_ipv6",
            (true, false) => "draft.families.ipv4_only",
            (false, true) => "draft.families.ipv6_only",
            (false, false) => "draft.families.none",
        }
    }

    pub fn peer6_is_link_local(&self) -> bool {
        self.peer6.trim().to_lowercase().starts_with("fe80:")
    }

    pub fn link_local_collision_with(&self, fallback_own6: Option<&str>) -> bool {
        let (_, peer6, own6) = self.tunnel_fields();
        link_local_addresses_collide(peer6.as_deref(), own6.as_deref().or(fallback_own6))
    }

    pub fn effective_mp_bgp_transport(&self) -> Option<MpBgpTransport> {
        let (peer4, peer6, _) = self.tunnel_fields();
        resolve_mp_bgp_transport(self.mp_bgp_transport, peer4.as_deref(), peer6.as_deref())
    }

    pub fn selected_mp_bgp_transport(&self) -> MpBgpTransport {
        self.effective_mp_bgp_transport().unwrap_or_default()
    }

    fn tunnel_fields(&self) -> (Option<String>, Option<String>, Option<String>) {
        (
            optional_string(&self.peer4),
            optional_string(&self.peer6),
            optional_string(&self.own6),
        )
    }

    fn peer4_requirement_error(
        &self,
        peer4: Option<&str>,
        peer6: Option<&str>,
    ) -> Option<&'static str> {
        let transport = resolve_mp_bgp_transport(self.mp_bgp_transport, peer4, peer6);
        let peer4_required_for_transport = self.mp_bgp && transport == Some(MpBgpTransport::Ipv4);
        let peer4_required_for_routes = self.ipv4 && !self.mp_bgp;

        if peer4.is_none() && peer4_required_for_transport {
            Some("validation.peer4.required_mp_bgp")
        } else if peer4.is_none() && peer4_required_for_routes {
            Some("validation.peer4.required_ipv4")
        } else {
            None
        }
    }

    fn peer6_requirement_error(
        &self,
        peer4: Option<&str>,
        peer6: Option<&str>,
    ) -> Option<&'static str> {
        let transport = resolve_mp_bgp_transport(self.mp_bgp_transport, peer4, peer6);
        let peer6_required_for_transport = self.mp_bgp && transport == Some(MpBgpTransport::Ipv6);
        let peer6_required_for_routes = self.ipv6 && !self.mp_bgp;

        if peer6.is_none() && peer6_required_for_transport {
            Some("validation.peer6.required_mp_bgp")
        } else if peer6.is_none() && peer6_required_for_routes {
            Some("validation.peer6.required_ipv6")
        } else {
            None
        }
    }

    fn bgp_dependency_error(
        &self,
        peer4: Option<&str>,
        peer6: Option<&str>,
    ) -> Option<&'static str> {
        let transport = resolve_mp_bgp_transport(self.mp_bgp_transport, peer4, peer6);
        if self.extended_next_hop && !self.mp_bgp {
            Some("validation.extended_next_hop.requires_mp_bgp")
        } else if self.extended_next_hop && !self.ipv4 {
            Some("validation.extended_next_hop.requires_ipv4")
        } else if self.extended_next_hop && transport != Some(MpBgpTransport::Ipv6) {
            Some("validation.extended_next_hop.requires_ipv6_transport")
        } else if self.ipv4
            && self.mp_bgp
            && transport == Some(MpBgpTransport::Ipv6)
            && peer4.is_none()
            && !self.extended_next_hop
        {
            Some("validation.ipv4_over_ipv6_transport.requires_peer4_or_enh")
        } else {
            None
        }
    }

    pub fn bgp_error(&self) -> Option<String> {
        let (peer4, peer6, _) = self.tunnel_fields();
        self.bgp_dependency_error(peer4.as_deref(), peer6.as_deref())
            .map(str::to_string)
    }

    fn own6_dependency_error(
        &self,
        own6: Option<&str>,
        peer6: Option<&str>,
    ) -> Option<&'static str> {
        if own6.is_some() && peer6.is_none() {
            Some("validation.own6.requires_peer6")
        } else if own6.is_some() && !self.peer6_is_link_local() {
            Some("validation.own6.requires_link_local_peer6")
        } else {
            None
        }
    }

    fn own6_prefix_error(own6: Option<&str>) -> Option<&'static str> {
        own6.filter(|value| !value.to_ascii_lowercase().starts_with("fe80:"))
            .map(|_| "validation.own6.must_start_fe80")
    }

    fn link_local_collision_error(peer6: Option<&str>, own6: Option<&str>) -> Option<&'static str> {
        link_local_addresses_collide(peer6, own6)
            .then_some("validation.own6.must_differ_from_peer6")
    }

    pub fn field_error(&self, field: SessionDraftField) -> Option<String> {
        let (peer4, peer6, own6) = self.tunnel_fields();

        match field {
            SessionDraftField::Endpoint => validate_endpoint(&self.endpoint).err(),
            SessionDraftField::WgPublicKey => {
                validate_wireguard_public_key(&self.wg_public_key).err()
            }
            SessionDraftField::Peer4 => self
                .peer4_requirement_error(peer4.as_deref(), peer6.as_deref())
                .map(str::to_string)
                .or_else(|| {
                    validate_peer_ipv4(peer4, "validation.peer4.invalid", "validation.peer4.range")
                        .err()
                }),
            SessionDraftField::Peer6 => self
                .peer6_requirement_error(peer4.as_deref(), peer6.as_deref())
                .map(str::to_string)
                .or_else(|| {
                    validate_peer_ipv6(
                        peer6.clone(),
                        "validation.peer6.invalid",
                        "validation.peer6.scope",
                    )
                    .err()
                })
                .or_else(|| {
                    Self::link_local_collision_error(peer6.as_deref(), own6.as_deref())
                        .map(str::to_string)
                }),
            SessionDraftField::Own6 => self
                .own6_dependency_error(own6.as_deref(), peer6.as_deref())
                .map(str::to_string)
                .or_else(|| Self::own6_prefix_error(own6.as_deref()).map(str::to_string))
                .or_else(|| {
                    validate_link_local_ipv6(
                        own6.clone(),
                        "validation.own6.invalid",
                        "validation.own6.scope",
                    )
                    .err()
                })
                .or_else(|| {
                    Self::link_local_collision_error(peer6.as_deref(), own6.as_deref())
                        .map(str::to_string)
                }),
            SessionDraftField::Keepalive => {
                optional_u16(&self.keepalive, "validation.keepalive.invalid").err()
            }
            SessionDraftField::Mtu => validate_optional_mtu(&self.mtu).err(),
        }
    }

    pub fn to_spec(&self) -> Result<PeerSessionSpec, String> {
        let (peer4, peer6, own6) = self.tunnel_fields();

        if peer4.is_none() && peer6.is_none() {
            return Err("validation.tunnel.required".to_string());
        }
        if !self.ipv4 && !self.ipv6 {
            return Err("validation.bgp_family.required".to_string());
        }
        if let Some(message) = self.peer6_requirement_error(peer4.as_deref(), peer6.as_deref()) {
            return Err(message.to_string());
        }
        if let Some(message) = self.peer4_requirement_error(peer4.as_deref(), peer6.as_deref()) {
            return Err(message.to_string());
        }
        if let Some(message) = self.bgp_dependency_error(peer4.as_deref(), peer6.as_deref()) {
            return Err(message.to_string());
        }
        if let Some(message) = self.own6_dependency_error(own6.as_deref(), peer6.as_deref()) {
            return Err(message.to_string());
        }

        let endpoint = validate_endpoint(&self.endpoint)?;
        let wg_public_key = validate_wireguard_public_key(&self.wg_public_key)?;
        let peer4 =
            validate_peer_ipv4(peer4, "validation.peer4.invalid", "validation.peer4.range")?;
        let peer6 =
            validate_peer_ipv6(peer6, "validation.peer6.invalid", "validation.peer6.scope")?;

        if let Some(message) = Self::own6_prefix_error(own6.as_deref()) {
            return Err(message.to_string());
        }
        let own6 =
            validate_link_local_ipv6(own6, "validation.own6.invalid", "validation.own6.scope")?;
        if link_local_addresses_collide(peer6.as_deref(), own6.as_deref()) {
            return Err("validation.own6.must_differ_from_peer6".to_string());
        }
        let mtu = validate_optional_mtu(&self.mtu)?;

        Ok(PeerSessionSpec {
            comment: (!self.comment.trim().is_empty()).then(|| self.comment.trim().to_string()),
            endpoint,
            wg_public_key,
            port: None,
            peer4,
            peer6,
            own6,
            keepalive: optional_u16(&self.keepalive, "validation.keepalive.invalid")?,
            mtu,
            ipv4: self.ipv4,
            ipv6: self.ipv6,
            extended_next_hop: self.extended_next_hop,
            mp_bgp: self.mp_bgp,
            mp_bgp_transport: self.mp_bgp.then_some(self.mp_bgp_transport).flatten(),
            peering_strategy: self.peering_strategy,
            psk: optional_string(&self.psk),
            encrypt_endpoint: if self.encrypt_endpoint { Some(true) } else { None },
        })
    }
}

fn resolve_mp_bgp_transport(
    explicit: Option<MpBgpTransport>,
    peer4: Option<&str>,
    peer6: Option<&str>,
) -> Option<MpBgpTransport> {
    explicit.or_else(|| {
        if peer6.is_some() {
            Some(MpBgpTransport::Ipv6)
        } else if peer4.is_some() {
            Some(MpBgpTransport::Ipv4)
        } else {
            None
        }
    })
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
        .get_item("bird-lg-rs.autopeer.sessions")
        .ok()
        .flatten()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_persisted_sessions(sessions: &PersistedSessions) {
    let Some(storage) = local_storage() else {
        return;
    };

    if sessions.auth_session.is_none() && sessions.host_session.is_none() {
        let _ = storage.remove_item("bird-lg-rs.autopeer.sessions");
        return;
    }

    if let Ok(value) = serde_json::to_string(sessions) {
        let _ = storage.set_item("bird-lg-rs.autopeer.sessions", &value);
    }
}

fn required(value: &str, error_key: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error_key.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn validate_endpoint(value: &str) -> Result<String, String> {
    let endpoint = required(value, "validation.endpoint.required")?;

    if endpoint.contains(char::is_whitespace) {
        return Err("validation.endpoint.no_spaces".to_string());
    }

    if let Some(host) = endpoint.strip_prefix('[') {
        let Some((ipv6, port)) = host.split_once("]:") else {
            return Err("validation.endpoint.ipv6_format".to_string());
        };
        parse_ipv6(ipv6, "validation.endpoint.ipv6_invalid")?;
        parse_port_component(port)?;
        return Ok(endpoint);
    }

    if endpoint.matches(':').count() != 1 {
        return Err("validation.endpoint.host_port_format".to_string());
    }

    let Some((host, port)) = endpoint.rsplit_once(':') else {
        return Err("validation.endpoint.port_required".to_string());
    };
    if host.is_empty() {
        return Err("validation.endpoint.host_required".to_string());
    }
    if host.parse::<Ipv4Addr>().is_err() && !is_valid_endpoint_hostname(host) {
        return Err("validation.endpoint.host_invalid".to_string());
    }
    parse_port_component(port)?;
    Ok(endpoint)
}

fn validate_wireguard_public_key(value: &str) -> Result<String, String> {
    let key = required(value, "validation.wg_public_key.required")?;

    if key.len() != 44 {
        return Err("validation.wg_public_key.length".to_string());
    }
    if !key.ends_with('=') {
        return Err("validation.wg_public_key.suffix".to_string());
    }
    if !key
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '+' | '/' | '='))
    {
        return Err("validation.wg_public_key.charset".to_string());
    }

    Ok(key)
}

fn validate_peer_ipv4(
    value: Option<String>,
    invalid_key: &str,
    range_key: &str,
) -> Result<Option<String>, String> {
    match value {
        Some(value) => {
            let addr = parse_ipv4(&value, invalid_key)?;
            if !is_dn42_tunnel_ipv4(addr) {
                return Err(range_key.to_string());
            }
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn validate_peer_ipv6(
    value: Option<String>,
    invalid_key: &str,
    scope_key: &str,
) -> Result<Option<String>, String> {
    match value {
        Some(value) => {
            let addr = parse_ipv6(&value, invalid_key)?;
            if !is_link_local_ipv6(addr) && !is_unique_local_ipv6(addr) {
                return Err(scope_key.to_string());
            }
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn validate_link_local_ipv6(
    value: Option<String>,
    invalid_key: &str,
    scope_key: &str,
) -> Result<Option<String>, String> {
    match value {
        Some(value) => {
            let addr = parse_ipv6(&value, invalid_key)?;
            if !is_link_local_ipv6(addr) {
                return Err(scope_key.to_string());
            }
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn validate_optional_mtu(value: &str) -> Result<Option<u16>, String> {
    let mtu = optional_u16(value, "validation.mtu.invalid")?;
    if let Some(mtu) = mtu
        && !(1280..=1500).contains(&mtu)
    {
        return Err("validation.mtu.range".to_string());
    }
    Ok(mtu)
}

fn parse_ipv4(value: &str, invalid_key: &str) -> Result<Ipv4Addr, String> {
    value
        .parse::<Ipv4Addr>()
        .map_err(|_| invalid_key.to_string())
}

fn parse_ipv6(value: &str, invalid_key: &str) -> Result<Ipv6Addr, String> {
    value
        .parse::<Ipv6Addr>()
        .map_err(|_| invalid_key.to_string())
}

fn parse_port_component(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| "validation.endpoint.port.invalid".to_string())
        .and_then(|port| {
            if port == 0 {
                Err("validation.endpoint.port.range".to_string())
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

fn parse_optional_link_local_ipv6(value: Option<&str>) -> Option<Ipv6Addr> {
    value
        .and_then(|value| value.trim().parse::<Ipv6Addr>().ok())
        .filter(|value| is_link_local_ipv6(*value))
}

fn link_local_addresses_collide(peer6: Option<&str>, own6: Option<&str>) -> bool {
    let Some(peer6_address) = parse_optional_link_local_ipv6(peer6) else {
        return false;
    };
    let Some(own6_address) = parse_optional_link_local_ipv6(own6) else {
        return false;
    };
    peer6_address == own6_address
}

fn optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn optional_u16(value: &str, error_key: &str) -> Result<Option<u16>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    trimmed
        .parse::<u16>()
        .map(Some)
        .map_err(|_| error_key.to_string())
}

#[cfg(test)]
mod tests {
    use super::{PersistedSessions, SessionDraft, SessionDraftField};
    use crate::models::{
        AuthMethod, AuthMethodKind, AuthSessionResponse, MpBgpTransport, PeerSessionSpec,
        PeeringStrategy, UiMessage,
    };

    const VALID_WG_KEY: &str = "sLbzTRr2gfLFb24NPzDOpy8j09Y6zI+a7NkeVMdVSR8=";

    #[test]
    fn draft_to_spec_trims_optional_fields() {
        let draft = SessionDraft {
            endpoint: " peer.example.net:21023 ".into(),
            wg_public_key: format!(" {VALID_WG_KEY} "),
            peer6: " fe80::1234 ".into(),
            ..SessionDraft::default()
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.endpoint, "peer.example.net:21023");
        assert_eq!(spec.wg_public_key, VALID_WG_KEY);
        assert_eq!(spec.peer6, Some("fe80::1234".into()));
        assert_eq!(spec.port, None);
        assert_eq!(spec.peering_strategy, PeeringStrategy::FullTable);
    }

    #[test]
    fn draft_to_spec_preserves_non_default_peering_strategy() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            peering_strategy: PeeringStrategy::Downstream,
            ..SessionDraft::default()
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
            ..SessionDraft::default()
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.peer4, Some("172.20.193.67".into()));
        assert_eq!(spec.peer6, Some("fd55:dead:beef::3".into()));
    }

    #[test]
    fn draft_to_spec_allows_ipv4_only_sessions() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp: false,
            ..SessionDraft::default()
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.peer4, Some("172.20.193.67".into()));
        assert_eq!(spec.peer6, None);
        assert!(spec.ipv4);
        assert!(!spec.ipv6);
        assert!(!spec.mp_bgp);
    }

    #[test]
    fn draft_to_spec_allows_ipv4_routes_over_ipv6_mp_bgp() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            ..SessionDraft::default()
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.peer4, None);
        assert_eq!(spec.peer6, Some("fd55:dead:beef::3".into()));
        assert!(spec.ipv4);
        assert!(!spec.ipv6);
        assert!(spec.mp_bgp);
    }

    #[test]
    fn draft_to_spec_allows_ipv6_routes_over_ipv4_mp_bgp_without_peer6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            extended_next_hop: false,
            ..SessionDraft::default()
        };

        let spec = draft.to_spec().unwrap();
        assert_eq!(spec.peer4, Some("172.20.193.67".into()));
        assert_eq!(spec.peer6, None);
        assert!(spec.ipv6);
        assert!(spec.mp_bgp);
        assert_eq!(spec.mp_bgp_transport, None);
    }

    #[test]
    fn draft_to_spec_requires_peer6_for_explicit_ipv6_mp_bgp_transport() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.peer6.required_mp_bgp"
        );
    }

    #[test]
    fn draft_to_spec_rejects_ipv6_routes_without_peer6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            ipv4: false,
            extended_next_hop: false,
            mp_bgp: false,
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.peer6.required_ipv6"
        );
    }

    #[test]
    fn draft_to_spec_rejects_ipv4_routes_without_peer4() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp: false,
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.peer4.required_ipv4"
        );
    }

    #[test]
    fn draft_to_spec_requires_peer4_for_explicit_ipv4_mp_bgp_transport() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv4: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv4),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.peer4.required_mp_bgp"
        );
    }

    #[test]
    fn draft_to_spec_rejects_ipv4_over_ipv6_transport_without_peer4_or_enh() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.ipv4_over_ipv6_transport.requires_peer4_or_enh"
        );
    }

    #[test]
    fn draft_to_spec_rejects_extended_next_hop_without_ipv4_family() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv4: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.extended_next_hop.requires_ipv4"
        );
    }

    #[test]
    fn draft_to_spec_rejects_extended_next_hop_over_ipv4_transport() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            peer6: "fd55:dead:beef::3".into(),
            mp_bgp_transport: Some(MpBgpTransport::Ipv4),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.extended_next_hop.requires_ipv6_transport"
        );
    }

    #[test]
    fn draft_to_spec_rejects_invalid_wireguard_key() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: "not-a-key".into(),
            peer6: "fe80::1234".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.wg_public_key.length"
        );
    }

    #[test]
    fn draft_to_spec_rejects_invalid_ipv4_address() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.999.67".into(),
            peer6: "fd55:dead:beef::3".into(),
            ..SessionDraft::default()
        };

        assert_eq!(draft.to_spec().unwrap_err(), "validation.peer4.invalid");
    }

    #[test]
    fn draft_to_spec_rejects_invalid_endpoint_shape() {
        let draft = SessionDraft {
            endpoint: "peer.example.net".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.endpoint.host_port_format"
        );
    }

    #[test]
    fn draft_to_spec_rejects_non_fqdn_endpoint_host() {
        let draft = SessionDraft {
            endpoint: "1:2".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.endpoint.host_invalid"
        );
    }

    #[test]
    fn draft_to_spec_rejects_peer4_outside_dn42_range() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "0.0.0.0".into(),
            peer6: "fd55:dead:beef::3".into(),
            ..SessionDraft::default()
        };

        assert_eq!(draft.to_spec().unwrap_err(), "validation.peer4.range");
    }

    #[test]
    fn draft_to_spec_rejects_peer6_outside_ula_and_link_local() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "::".into(),
            ..SessionDraft::default()
        };

        assert_eq!(draft.to_spec().unwrap_err(), "validation.peer6.scope");
    }

    #[test]
    fn draft_to_spec_rejects_out_of_range_mtu() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            mtu: "11451".into(),
            ..SessionDraft::default()
        };

        assert_eq!(draft.to_spec().unwrap_err(), "validation.mtu.range");
    }

    #[test]
    fn draft_to_spec_rejects_non_link_local_own6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            own6: "fd42::1".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.own6.must_start_fe80"
        );
    }

    #[test]
    fn draft_to_spec_rejects_colliding_link_local_addresses() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            own6: "fe80::1234".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.to_spec().unwrap_err(),
            "validation.own6.must_differ_from_peer6"
        );
    }

    #[test]
    fn field_error_requires_peer4_for_ipv4_without_mp_bgp() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp: false,
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.field_error(SessionDraftField::Peer4),
            Some("validation.peer4.required_ipv4".into())
        );
    }

    #[test]
    fn field_error_requires_peer6_before_link_local_own6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            own6: "fe80::1".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.field_error(SessionDraftField::Own6),
            Some("validation.own6.requires_peer6".into())
        );
    }

    #[test]
    fn field_error_marks_both_fields_for_colliding_link_local_addresses() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fe80::1234".into(),
            own6: "fe80::1234".into(),
            ..SessionDraft::default()
        };

        assert_eq!(
            draft.field_error(SessionDraftField::Peer6),
            Some("validation.own6.must_differ_from_peer6".into())
        );
        assert_eq!(
            draft.field_error(SessionDraftField::Own6),
            Some("validation.own6.must_differ_from_peer6".into())
        );
    }

    #[test]
    fn field_error_matches_to_spec_for_missing_peer6_on_ipv6_transport() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            ipv6: false,
            extended_next_hop: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv6),
            ..SessionDraft::default()
        };

        let expected = "validation.peer6.required_mp_bgp";
        assert_eq!(
            draft.field_error(SessionDraftField::Peer6),
            Some(expected.into())
        );
        assert_eq!(draft.to_spec().unwrap_err(), expected);
    }

    #[test]
    fn field_error_matches_to_spec_for_missing_peer4_on_ipv4_transport() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv4: false,
            mp_bgp_transport: Some(MpBgpTransport::Ipv4),
            ..SessionDraft::default()
        };

        let expected = "validation.peer4.required_mp_bgp";
        assert_eq!(
            draft.field_error(SessionDraftField::Peer4),
            Some(expected.into())
        );
        assert_eq!(draft.to_spec().unwrap_err(), expected);
    }

    #[test]
    fn field_error_uses_ipv4_transport_for_dual_family_mp_bgp_without_peer6() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer4: "172.20.193.67".into(),
            extended_next_hop: false,
            ..SessionDraft::default()
        };

        assert_eq!(draft.field_error(SessionDraftField::Peer6), None);
        assert!(draft.to_spec().is_ok());
    }

    #[test]
    fn field_error_uses_ipv6_transport_for_dual_family_mp_bgp_without_peer4() {
        let draft = SessionDraft {
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            peer6: "fd55:dead:beef::3".into(),
            ipv6: false,
            ..SessionDraft::default()
        };

        assert_eq!(draft.field_error(SessionDraftField::Peer4), None);
        assert!(draft.to_spec().is_ok());
    }

    #[test]
    fn field_error_roundtrip_preserves_legacy_mp_bgp_transport_omission() {
        let spec = PeerSessionSpec {
            comment: Some("peer note".into()),
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            port: None,
            peer4: Some("172.20.193.67".into()),
            peer6: Some("fe80::3".into()),
            own6: Some("fe80::1".into()),
            keepalive: Some(25),
            mtu: Some(1420),
            ipv4: true,
            ipv6: true,
            extended_next_hop: true,
            mp_bgp: true,
            mp_bgp_transport: None,
            peering_strategy: PeeringStrategy::Transit,
            psk: None,
            encrypt_endpoint: None,
        };

        let roundtrip = SessionDraft::from_session("lax-01", &spec)
            .to_spec()
            .unwrap();
        assert_eq!(roundtrip, spec);
    }

    #[test]
    fn from_session_roundtrip_preserves_flags_and_strategy() {
        let spec = PeerSessionSpec {
            comment: Some("peer note".into()),
            endpoint: "peer.example.net:21023".into(),
            wg_public_key: VALID_WG_KEY.into(),
            port: None,
            peer4: Some("172.20.193.67".into()),
            peer6: Some("fe80::3".into()),
            own6: Some("fe80::1".into()),
            keepalive: Some(25),
            mtu: Some(1420),
            ipv4: true,
            ipv6: true,
            extended_next_hop: false,
            mp_bgp: false,
            mp_bgp_transport: None,
            peering_strategy: PeeringStrategy::Transit,
            psk: None,
            encrypt_endpoint: None,
        };

        let roundtrip = SessionDraft::from_session("lax-01", &spec)
            .to_spec()
            .unwrap();
        assert_eq!(roundtrip, spec);
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
                    label: UiMessage::raw("Registry SSH Signature"),
                    description: UiMessage::raw("Signed with maintainer SSH key"),
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
