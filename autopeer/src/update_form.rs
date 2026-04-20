use std::{collections::BTreeSet, net::Ipv6Addr};

use crate::store::{SessionDraft, SessionDraftField};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionDraftToggleGroup {
    Families,
    Bgp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionDraftTouchedControl {
    Field(SessionDraftField),
    ToggleGroup(SessionDraftToggleGroup),
}

impl From<SessionDraftField> for SessionDraftTouchedControl {
    fn from(field: SessionDraftField) -> Self {
        Self::Field(field)
    }
}

impl From<SessionDraftToggleGroup> for SessionDraftTouchedControl {
    fn from(group: SessionDraftToggleGroup) -> Self {
        Self::ToggleGroup(group)
    }
}

pub type SessionDraftTouchedControls = BTreeSet<SessionDraftTouchedControl>;

pub fn touch_field(touched_controls: &mut SessionDraftTouchedControls, field: SessionDraftField) {
    touched_controls.insert(field.into());
}

pub fn touch_toggle_group(
    touched_controls: &mut SessionDraftTouchedControls,
    group: SessionDraftToggleGroup,
) {
    touched_controls.insert(group.into());
}

pub fn field_is_touched(
    touched_controls: &SessionDraftTouchedControls,
    field: SessionDraftField,
) -> bool {
    touched_controls.contains(&field.into())
}

pub fn toggle_group_is_touched(
    touched_controls: &SessionDraftTouchedControls,
    group: SessionDraftToggleGroup,
) -> bool {
    touched_controls.contains(&group.into())
}

fn control_is_focused(focused_field: Option<SessionDraftField>, field: SessionDraftField) -> bool {
    focused_field == Some(field)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionDraftLiveValidation {
    pub peer4_message: Option<String>,
    pub peer6_messages: Vec<String>,
    pub own6_message: Option<String>,
    pub tunnel_message: Option<String>,
    pub families_message: Option<String>,
    pub bgp_message: Option<String>,
    pub highlight_peer4: bool,
    pub highlight_peer6: bool,
    pub highlight_own6: bool,
    pub highlight_ipv4: bool,
    pub highlight_ipv6: bool,
    pub highlight_mp_bgp: bool,
    pub highlight_extended_next_hop: bool,
}

impl SessionDraftLiveValidation {
    pub fn highlights_field(&self, field: SessionDraftField) -> bool {
        match field {
            SessionDraftField::Peer4 => self.highlight_peer4,
            SessionDraftField::Peer6 => self.highlight_peer6,
            SessionDraftField::Own6 => self.highlight_own6,
            _ => false,
        }
    }
}

fn matches_validation_key(message: Option<&str>, expected: &str) -> bool {
    matches!(message, Some(value) if value == expected)
}

pub fn session_details_live_validation(
    draft: &SessionDraft,
    touched_controls: &SessionDraftTouchedControls,
    focused_field: Option<SessionDraftField>,
    fallback_own6: Option<&str>,
) -> SessionDraftLiveValidation {
    let peer4_touched = field_is_touched(touched_controls, SessionDraftField::Peer4);
    let peer6_touched = field_is_touched(touched_controls, SessionDraftField::Peer6);
    let own6_touched = field_is_touched(touched_controls, SessionDraftField::Own6);
    let peer4_focused = control_is_focused(focused_field, SessionDraftField::Peer4);
    let peer6_focused = control_is_focused(focused_field, SessionDraftField::Peer6);
    let own6_focused = control_is_focused(focused_field, SessionDraftField::Own6);
    let peer4_error = draft.field_error(SessionDraftField::Peer4);
    let peer6_error = draft.field_error(SessionDraftField::Peer6);
    let bgp_error = draft.bgp_error();
    let families_touched =
        toggle_group_is_touched(touched_controls, SessionDraftToggleGroup::Families);
    let bgp_touched = toggle_group_is_touched(touched_controls, SessionDraftToggleGroup::Bgp);
    let combo_touched = families_touched || bgp_touched;

    let peer4_blank = draft.peer4.trim().is_empty();
    let peer6_blank = draft.peer6.trim().is_empty();
    let own6_present = !draft.own6.trim().is_empty();

    let no_families_selected = !draft.ipv4 && !draft.ipv6;
    let peer4_missing_for_ipv4 = draft.ipv4 && !draft.mp_bgp && peer4_blank;
    let peer6_missing_for_ipv6 = draft.ipv6 && !draft.mp_bgp && peer6_blank;
    let link_local_own6_collision = draft.link_local_collision_with(fallback_own6);
    let show_generic_tunnel_required = !own6_present
        && peer4_blank
        && peer6_blank
        && (peer4_touched || peer6_touched || own6_touched || combo_touched)
        && !peer4_focused
        && !peer6_focused;
    let peer4_requirement_touched = peer4_touched || families_touched || bgp_touched;
    let peer6_requirement_touched = peer6_touched || families_touched || bgp_touched;

    let peer4_message = if !peer4_focused
        && !show_generic_tunnel_required
        && ((peer4_blank && peer4_requirement_touched && peer4_error.is_some())
            || (!peer4_blank && peer4_touched && peer4_error.is_some()))
    {
        peer4_error.clone()
    } else {
        None
    };
    let peer6_messages = if link_local_own6_collision || show_generic_tunnel_required {
        Vec::new()
    } else if peer6_blank {
        if peer6_focused && !peer6_touched {
            Vec::new()
        } else if peer6_requirement_touched {
            peer6_error.clone().into_iter().collect()
        } else {
            Vec::new()
        }
    } else if peer6_focused {
        Vec::new()
    } else if !peer6_blank && peer6_touched {
        peer6_error.clone().into_iter().collect()
    } else {
        Vec::new()
    };
    let own6_message = if link_local_own6_collision && !own6_focused {
        Some("validation.own6.must_differ_from_peer6".to_string())
    } else if (own6_present || own6_touched) && !own6_focused {
        draft.field_error(SessionDraftField::Own6)
    } else {
        None
    };
    let peer4_highlight = !peer4_focused && !peer4_blank && peer4_error.is_some();
    let peer6_highlight =
        !peer6_focused && ((!peer6_blank && peer6_error.is_some()) || link_local_own6_collision);
    let bgp_message =
        if show_generic_tunnel_required || !(bgp_touched || peer4_touched || peer6_touched) {
            None
        } else {
            bgp_error.clone()
        };

    SessionDraftLiveValidation {
        peer4_message,
        peer6_messages,
        own6_message: own6_message.clone(),
        tunnel_message: show_generic_tunnel_required
            .then(|| "validation.tunnel.required".to_string()),
        families_message: (families_touched && no_families_selected)
            .then(|| "validation.bgp_family.required".to_string()),
        bgp_message: bgp_message.clone(),
        highlight_peer4: peer4_highlight,
        highlight_peer6: peer6_highlight,
        highlight_own6: own6_message.is_some(),
        highlight_ipv4: (families_touched && no_families_selected)
            || (!show_generic_tunnel_required
                && !peer4_focused
                && peer4_missing_for_ipv4
                && families_touched)
            || matches_validation_key(
                bgp_message.as_deref(),
                "validation.extended_next_hop.requires_ipv4",
            ),
        highlight_ipv6: (families_touched && no_families_selected)
            || (!show_generic_tunnel_required
                && !peer6_focused
                && peer6_missing_for_ipv6
                && families_touched),
        highlight_mp_bgp: matches_validation_key(
            bgp_message.as_deref(),
            "validation.extended_next_hop.requires_mp_bgp",
        ),
        highlight_extended_next_hop: !peer6_focused
            && matches!(
                bgp_message.as_deref(),
                Some(
                    "validation.extended_next_hop.requires_mp_bgp"
                        | "validation.extended_next_hop.requires_ipv4"
                        | "validation.extended_next_hop.requires_ipv6_transport"
                        | "validation.ipv4_over_ipv6_transport.requires_peer4_or_enh"
                )
            ),
    }
}

pub fn session_details_submission_error(
    draft: &SessionDraft,
    fallback_own6: Option<&str>,
) -> Option<String> {
    if draft.link_local_collision_with(fallback_own6) {
        Some("validation.own6.must_differ_from_peer6".to_string())
    } else {
        draft.to_spec().err()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peer6AddressKind {
    LinkLocal,
    Ula,
}

pub fn detect_peer6_address_kind(value: &str) -> Option<Peer6AddressKind> {
    let parsed = value.trim().parse::<Ipv6Addr>().ok()?;
    if parsed.segments()[0] == 0xfe80 {
        Some(Peer6AddressKind::LinkLocal)
    } else if (parsed.segments()[0] & 0xfe00) == 0xfc00 {
        Some(Peer6AddressKind::Ula)
    } else {
        None
    }
}

pub fn displayed_peer6_address_kind(
    current_value: &str,
    focused_field: Option<SessionDraftField>,
    committed_kind: Option<Peer6AddressKind>,
) -> Option<Peer6AddressKind> {
    if focused_field == Some(SessionDraftField::Peer6) {
        committed_kind
    } else {
        detect_peer6_address_kind(current_value)
    }
}

pub fn should_display_node_ipv4(draft: &SessionDraft) -> bool {
    !draft.peer4.trim().is_empty() && draft.field_error(SessionDraftField::Peer4).is_none()
}

pub fn displayed_node_ipv4_visibility(
    draft: &SessionDraft,
    focused_field: Option<SessionDraftField>,
    committed_visibility: bool,
) -> bool {
    if focused_field == Some(SessionDraftField::Peer4) {
        committed_visibility
    } else {
        should_display_node_ipv4(draft)
    }
}

pub fn should_mark_field_invalid(draft: &SessionDraft, field: SessionDraftField) -> bool {
    let value = match field {
        SessionDraftField::Endpoint => draft.endpoint.as_str(),
        SessionDraftField::WgPublicKey => draft.wg_public_key.as_str(),
        SessionDraftField::Peer4 => draft.peer4.as_str(),
        SessionDraftField::Peer6 => draft.peer6.as_str(),
        SessionDraftField::Own6 => draft.own6.as_str(),
        SessionDraftField::Keepalive => draft.keepalive.as_str(),
        SessionDraftField::Mtu => draft.mtu.as_str(),
    };

    !value.trim().is_empty() && draft.field_error(field).is_some()
}
