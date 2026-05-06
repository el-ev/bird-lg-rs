use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use super::generate_wg_psk;
use crate::{
    models::{MpBgpTransport, PeeringStrategy, UiMessage},
    store::{PeerConfigStage, SessionDraft, SessionDraftField},
    update_form::{
        Peer6AddressKind, SessionDraftToggleGroup, SessionDraftTouchedControls,
        detect_peer6_address_kind, session_details_submission_error,
    },
};

fn update_draft_state(
    draft: &UseStateHandle<SessionDraft>,
    update: impl FnOnce(&mut SessionDraft),
) {
    let mut next = (**draft).clone();
    update(&mut next);
    draft.set(next);
}

fn update_touched_controls(
    touched_fields: &UseStateHandle<SessionDraftTouchedControls>,
    update: impl FnOnce(&mut SessionDraftTouchedControls),
) {
    let mut next = (**touched_fields).clone();
    update(&mut next);
    touched_fields.set(next);
}

pub struct ManageCallbacks {
    pub on_cancel_edit: Callback<MouseEvent>,
    pub on_comment_change: Callback<String>,
    pub on_field_blur: Callback<SessionDraftField>,
    pub on_field_focus: Callback<SessionDraftField>,
    pub on_peer6_blur: Callback<FocusEvent>,
    pub on_peer6_change: Callback<String>,
    pub on_text_field_change: Callback<(SessionDraftField, String)>,
    pub on_toggle_ipv4: Callback<()>,
    pub on_toggle_ipv6: Callback<()>,
    pub on_toggle_mp_bgp: Callback<()>,
    pub on_toggle_extended_next_hop: Callback<()>,
    pub on_change_mp_bgp_transport: Callback<Event>,
    pub on_change_peering_strategy: Callback<Event>,
    pub on_toggle_encrypt_endpoint: Callback<()>,
    pub on_psk_action: Callback<MouseEvent>,
    pub on_back_to_details: Callback<MouseEvent>,
    pub on_change_node: Callback<MouseEvent>,
    pub on_step_click: Callback<PeerConfigStage>,
    pub on_continue_to_review: Callback<MouseEvent>,
}

pub fn build_manage_callbacks(
    draft: &UseStateHandle<SessionDraft>,
    touched_fields: &UseStateHandle<SessionDraftTouchedControls>,
    editing_node: &UseStateHandle<Option<String>>,
    config_stage: &UseStateHandle<PeerConfigStage>,
    focused_field: &UseStateHandle<Option<SessionDraftField>>,
    committed_peer6_kind: &UseStateHandle<Option<Peer6AddressKind>>,
    sessions: &[crate::models::SessionView],
    nodes: &[crate::models::NodeView],
    error: &UseStateHandle<Option<UiMessage>>,
    psk_copied: &UseStateHandle<bool>,
    node_inventory_link_local_ipv6: Option<String>,
) -> ManageCallbacks {
    let on_cancel_edit = {
        let editing_node = editing_node.clone();
        let draft = draft.clone();
        let sessions = sessions.to_vec();
        let nodes = nodes.to_vec();
        let config_stage = config_stage.clone();
        let touched_fields = touched_fields.clone();
        Callback::from(move |_| {
            editing_node.set(None);
            config_stage.set(PeerConfigStage::SelectNode);
            touched_fields.set(SessionDraftTouchedControls::new());
            draft.set(crate::controller::sync_create_draft(
                &nodes, &sessions, &draft,
            ));
        })
    };

    let on_comment_change = {
        let draft = draft.clone();
        Callback::from(move |value: String| {
            update_draft_state(&draft, |next| next.comment = value);
        })
    };

    let on_text_field_change = {
        let draft = draft.clone();
        Callback::from(move |(field, value): (SessionDraftField, String)| {
            update_draft_state(&draft, |next| {
                let target = match field {
                    SessionDraftField::Endpoint => &mut next.endpoint,
                    SessionDraftField::WgPublicKey => &mut next.wg_public_key,
                    SessionDraftField::Peer4 => &mut next.peer4,
                    SessionDraftField::Peer6 => &mut next.peer6,
                    SessionDraftField::Own6 => &mut next.own6,
                    SessionDraftField::Keepalive => &mut next.keepalive,
                    SessionDraftField::Mtu => &mut next.mtu,
                    SessionDraftField::Psk => &mut next.psk,
                };
                *target = value;
            });
        })
    };

    let on_field_blur = {
        let touched_fields = touched_fields.clone();
        let focused_field = focused_field.clone();
        Callback::from(move |field: SessionDraftField| {
            if *focused_field == Some(field) {
                focused_field.set(None);
            }
            update_touched_controls(&touched_fields, |next| {
                next.insert(field.into());
            });
        })
    };

    let on_field_focus = {
        let focused_field = focused_field.clone();
        Callback::from(move |field: SessionDraftField| focused_field.set(Some(field)))
    };

    let on_peer6_blur = {
        let touched_fields = touched_fields.clone();
        let focused_field = focused_field.clone();
        let committed_peer6_kind = committed_peer6_kind.clone();
        let draft = draft.clone();
        Callback::from(move |_: FocusEvent| {
            if *focused_field == Some(SessionDraftField::Peer6) {
                focused_field.set(None);
            }
            update_touched_controls(&touched_fields, |next| {
                next.insert(SessionDraftField::Peer6.into());
            });
            let next_kind = detect_peer6_address_kind(&draft.peer6);
            committed_peer6_kind.set(next_kind);
            if next_kind != Some(Peer6AddressKind::LinkLocal) && !draft.own6.is_empty() {
                update_draft_state(&draft, |next| next.own6.clear());
            }
        })
    };

    let on_peer6_change = {
        let draft = draft.clone();
        Callback::from(move |value: String| {
            update_draft_state(&draft, |next| next.peer6 = value);
        })
    };

    let on_toggle_ipv4 = {
        let draft = draft.clone();
        let touched_fields = touched_fields.clone();
        Callback::from(move |_| {
            update_touched_controls(&touched_fields, |next| {
                next.insert(SessionDraftToggleGroup::Families.into());
                next.insert(SessionDraftToggleGroup::Bgp.into());
            });
            update_draft_state(&draft, |next| next.ipv4 = !next.ipv4);
        })
    };

    let on_toggle_ipv6 = {
        let draft = draft.clone();
        let touched_fields = touched_fields.clone();
        Callback::from(move |_| {
            update_touched_controls(&touched_fields, |next| {
                next.insert(SessionDraftToggleGroup::Families.into());
            });
            update_draft_state(&draft, |next| next.ipv6 = !next.ipv6);
        })
    };

    let on_toggle_mp_bgp = {
        let draft = draft.clone();
        let touched_fields = touched_fields.clone();
        Callback::from(move |_: ()| {
            update_touched_controls(&touched_fields, |next| {
                next.insert(SessionDraftToggleGroup::Bgp.into());
            });
            update_draft_state(&draft, |next| {
                next.mp_bgp = !next.mp_bgp;
                if !next.mp_bgp {
                    next.extended_next_hop = false;
                }
            });
        })
    };

    let on_toggle_extended_next_hop = {
        let draft = draft.clone();
        let touched_fields = touched_fields.clone();
        Callback::from(move |_| {
            update_touched_controls(&touched_fields, |next| {
                next.insert(SessionDraftToggleGroup::Bgp.into());
            });
            update_draft_state(&draft, |next| {
                next.extended_next_hop = !next.extended_next_hop;
                if next.extended_next_hop {
                    next.mp_bgp = true;
                    next.ipv4 = true;
                    next.mp_bgp_transport = Some(MpBgpTransport::Ipv6);
                }
            });
        })
    };

    let on_change_mp_bgp_transport = {
        let draft = draft.clone();
        let touched_fields = touched_fields.clone();
        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            let value = select.value();
            update_touched_controls(&touched_fields, |next| {
                next.insert(SessionDraftToggleGroup::Bgp.into());
            });
            update_draft_state(&draft, |next| {
                next.mp_bgp_transport = MpBgpTransport::from_value(&value);
                if next.mp_bgp_transport == Some(MpBgpTransport::Ipv4) {
                    next.extended_next_hop = false;
                }
            });
        })
    };

    let on_change_peering_strategy = {
        let draft = draft.clone();
        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            let value = select.value();
            update_draft_state(&draft, |next| {
                next.peering_strategy =
                    PeeringStrategy::from_value(&value).unwrap_or(PeeringStrategy::FullTable);
            });
        })
    };

    let on_toggle_encrypt_endpoint = {
        let draft = draft.clone();
        Callback::from(move |_| {
            update_draft_state(&draft, |next| {
                next.encrypt_endpoint = !next.encrypt_endpoint
            });
        })
    };

    let on_psk_action = {
        let draft = draft.clone();
        let psk_copied = psk_copied.clone();
        Callback::from(move |_: MouseEvent| {
            if draft.has_psk {
                update_draft_state(&draft, |next| {
                    next.clear_psk = true;
                    next.has_psk = false;
                    next.psk.clear();
                });
            } else if !draft.psk.is_empty() {
                update_draft_state(&draft, |next| {
                    next.psk.clear();
                });
            } else if let Some(key) = generate_wg_psk() {
                let draft = draft.clone();
                let psk_copied = psk_copied.clone();
                update_draft_state(&draft, |next| {
                    next.psk = key.clone();
                });
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let psk_copied_inner = psk_copied.clone();
                    spawn_local(async move {
                        let _ =
                            wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&key)).await;
                        psk_copied_inner.set(true);
                        gloo_timers::future::TimeoutFuture::new(2_000).await;
                        psk_copied_inner.set(false);
                    });
                }
            }
        })
    };

    let on_back_to_details = {
        let config_stage = config_stage.clone();
        Callback::from(move |_| config_stage.set(PeerConfigStage::SessionDetails))
    };

    let on_change_node = {
        let config_stage = config_stage.clone();
        Callback::from(move |_| config_stage.set(PeerConfigStage::SelectNode))
    };

    let on_step_click = {
        let config_stage = config_stage.clone();
        Callback::from(move |stage: PeerConfigStage| config_stage.set(stage))
    };

    let on_continue_to_review = {
        let draft = draft.clone();
        let editing_node = editing_node.clone();
        let config_stage = config_stage.clone();
        let error = error.clone();
        Callback::from(move |_| {
            if editing_node.is_none() && draft.node.trim().is_empty() {
                error.set(Some(UiMessage::key("error.ui.node.choose")));
                return;
            }

            match session_details_submission_error(
                &draft,
                node_inventory_link_local_ipv6.as_deref(),
            ) {
                None => {
                    error.set(None);
                    config_stage.set(PeerConfigStage::Review);
                }
                Some(message) => error.set(Some(UiMessage::key(message))),
            }
        })
    };

    ManageCallbacks {
        on_cancel_edit,
        on_comment_change,
        on_field_blur,
        on_field_focus,
        on_peer6_blur,
        on_peer6_change,
        on_text_field_change,
        on_toggle_ipv4,
        on_toggle_ipv6,
        on_toggle_mp_bgp,
        on_toggle_extended_next_hop,
        on_change_mp_bgp_transport,
        on_change_peering_strategy,
        on_toggle_encrypt_endpoint,
        on_psk_action,
        on_back_to_details,
        on_change_node,
        on_step_click,
        on_continue_to_review,
    }
}
