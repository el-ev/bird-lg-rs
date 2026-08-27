use ui_components::shell::{
    ShellButton, ShellInput, ShellLine, ShellPrompt, ShellSelect, ShellToggle,
};
use yew::prelude::*;

use super::{
    help_hint, humanize_ip_support, live_validation_block, live_validation_block_multi,
    node_context_line, render_error, render_ongoing_tasks,
};
use crate::{
    controller::OngoingTask,
    i18n::I18n,
    models::{
        ALL_MP_BGP_TRANSPORTS, ALL_PEERING_STRATEGIES, NodeView, SessionState, SessionView,
        UiMessage,
    },
    store::{SessionDraft, SessionDraftField},
    update_form::{
        Peer6AddressKind, SessionDraftLiveValidation, SessionDraftTouchedControls,
        should_mark_field_invalid,
    },
};

fn toggle_item_class(invalid: bool) -> Classes {
    classes!("autopeer-toggle-item", invalid.then_some("is-invalid"))
}

#[derive(Properties, PartialEq)]
pub struct SessionDetailsProps {
    pub i18n: I18n,
    pub loading: bool,
    pub draft: SessionDraft,
    pub editing_node: Option<String>,
    pub selected_node: Option<NodeView>,
    pub selected_session: Option<SessionView>,
    pub live_validation: SessionDraftLiveValidation,
    pub touched_fields: SessionDraftTouchedControls,
    pub focused_field: Option<SessionDraftField>,
    pub show_node_ipv4: bool,
    pub peer6_kind: Option<Peer6AddressKind>,
    pub own6_placeholder: String,
    pub node_inventory_ipv4: Option<String>,
    pub node_inventory_ipv6: Option<String>,
    pub psk_copied: bool,
    pub retire_confirmation: bool,
    pub delete_confirmation: bool,
    pub draft_is_valid: bool,
    pub ongoing_tasks: Vec<OngoingTask>,
    pub error: Option<UiMessage>,
    pub on_cancel_edit: Callback<MouseEvent>,
    pub on_field_focus: Callback<SessionDraftField>,
    pub on_field_blur: Callback<SessionDraftField>,
    pub on_peer6_blur: Callback<FocusEvent>,
    pub on_text_field_change: Callback<(SessionDraftField, String)>,
    pub on_comment_change: Callback<String>,
    pub on_peer6_change: Callback<String>,
    pub on_toggle_ipv4: Callback<()>,
    pub on_toggle_ipv6: Callback<()>,
    pub on_toggle_mp_bgp: Callback<()>,
    pub on_toggle_extended_next_hop: Callback<()>,
    pub on_change_mp_bgp_transport: Callback<Event>,
    pub on_change_peering_strategy: Callback<Event>,
    pub on_toggle_encrypt_endpoint: Callback<()>,
    pub on_psk_action: Callback<MouseEvent>,
    pub on_change_node: Callback<MouseEvent>,
    pub on_continue_to_review: Callback<MouseEvent>,
    pub on_retire_selected_session: Callback<MouseEvent>,
    pub on_delete_selected_session: Callback<MouseEvent>,
    pub on_retry_operation: Callback<String>,
    pub on_drop_operation: Callback<String>,
}

impl SessionDetailsProps {
    fn field_is_invalid(&self, field: SessionDraftField) -> bool {
        self.touched_fields.contains(&field.into())
            && self.focused_field != Some(field)
            && should_mark_field_invalid(&self.draft, field)
    }

    fn input_class(&self, field: SessionDraftField) -> Classes {
        if self.field_is_invalid(field) || self.live_validation.highlights_field(field) {
            classes!("shell-input--invalid")
        } else {
            Classes::new()
        }
    }

    fn input_frame_class(&self, field: SessionDraftField) -> Classes {
        if self.field_is_invalid(field) || self.live_validation.highlights_field(field) {
            classes!("shell-input-frame--invalid")
        } else {
            Classes::new()
        }
    }

    fn field_validation_block(&self, field: SessionDraftField, content: Html) -> Html {
        let has_error = self.field_is_invalid(field);
        let message = if has_error {
            self.draft.field_error(field)
        } else {
            None
        };
        live_validation_block(&self.i18n, message.as_deref(), content)
    }

    fn update_field_cb(&self, field: SessionDraftField) -> Callback<String> {
        let cb = self.on_text_field_change.clone();
        Callback::from(move |value: String| cb.emit((field, value)))
    }

    fn on_focus_cb(&self, field: SessionDraftField) -> Callback<FocusEvent> {
        let cb = self.on_field_focus.clone();
        Callback::from(move |_: FocusEvent| cb.emit(field))
    }

    fn on_blur_cb(&self, field: SessionDraftField) -> Callback<FocusEvent> {
        let cb = self.on_field_blur.clone();
        Callback::from(move |_: FocusEvent| cb.emit(field))
    }
}

#[function_component(SessionDetailsPanel)]
pub fn session_details_panel(props: &SessionDetailsProps) -> Html {
    let i18n = &props.i18n;
    let draft = &props.draft;
    let lv = &props.live_validation;

    html! {
        <article class="peering-card autopeer-panel">
            <div class="autopeer-panel-header">
                <p class="autopeer-panel-kicker">{i18n.t("stage2.kicker")}</p>
                <h3 class="autopeer-panel-title">
                    {
                        if let Some(node) = &props.editing_node {
                            i18n.translate_params(
                                "stage2.title.update_prefix",
                                &[("node", node.as_str())],
                            )
                        } else if let Some(node) = &props.selected_node {
                            i18n.translate_params(
                                "stage2.title.create_prefix",
                                &[("node", node.name.as_str())],
                            )
                        } else {
                            i18n.t("stage2.title.create_blank").to_string()
                        }
                    }
                </h3>
                if props.editing_node.is_some() {
                    <p class="text-secondary">
                        {i18n.t("stage2.update_intro")}
                    </p>
                }
            </div>

            if let Some(node) = &props.selected_node {
                <div class="autopeer-node-summary">
                    <div>
                        <strong>{node_context_line(i18n, node)}</strong>
                        if let Some(comment) = &node.comment {
                            <p class="text-secondary">{comment.clone()}</p>
                        }
                    </div>
                    <ShellButton text={i18n.t("action.choose_another_node")} onclick={if props.editing_node.is_some() { props.on_cancel_edit.clone() } else { props.on_change_node.clone() }} disabled={props.loading} />
                </div>
            }

            if let Some(session) = props.selected_session.as_ref().filter(|s| s.state == SessionState::StalledPr) {
                <div class="autopeer-stalled-banner">
                    <p class="autopeer-stalled-banner-title">{i18n.t("stalled.banner.title")}</p>
                    <p class="text-secondary">{i18n.t("stalled.banner.body")}</p>
                    <div class="autopeer-links">
                        if let Some(pr_url) = &session.pull_request_url {
                            <a href={pr_url.clone()} target="_blank" rel="noreferrer">{i18n.t("action.open_pr")}</a>
                        }
                        if let Some(op_id) = &session.pending_operation_id {
                            <button
                                class="autopeer-link-button"
                                onclick={props.on_retry_operation.reform({
                                    let op_id = op_id.clone();
                                    move |_: MouseEvent| op_id.clone()
                                })}
                                disabled={props.loading}
                            >
                                {i18n.t("action.redeploy")}
                            </button>
                            <button
                                class="autopeer-link-button autopeer-link-button--muted"
                                onclick={props.on_drop_operation.reform({
                                    let op_id = op_id.clone();
                                    move |_: MouseEvent| op_id.clone()
                                })}
                                disabled={props.loading}
                            >
                                {i18n.t("action.drop_changes")}
                            </button>
                        }
                    </div>
                </div>
            }

            <div class="autopeer-form-section">
                <span class="autopeer-section-label">{i18n.t("stage2.section.connection")}</span>
                <ShellLine>
                    <ShellPrompt>{i18n.t("stage2.field.endpoint")}</ShellPrompt>
                    {" "}
                    <ShellInput
                        value={draft.endpoint.clone()}
                        on_change={props.update_field_cb(SessionDraftField::Endpoint)}
                        class={props.input_class(SessionDraftField::Endpoint)}
                        frame_class={props.input_frame_class(SessionDraftField::Endpoint)}
                        on_focus={props.on_focus_cb(SessionDraftField::Endpoint)}
                        on_blur={props.on_blur_cb(SessionDraftField::Endpoint)}
                        placeholder={i18n.t("stage2.field.endpoint.placeholder")}
                        disabled={props.loading}
                    />
                    {" "}
                    <ShellToggle
                        active={draft.encrypt_endpoint}
                        on_toggle={props.on_toggle_encrypt_endpoint.clone()}
                        label={i18n.t("stage2.field.encrypt_endpoint")}
                        disabled={props.loading}
                    />
                    {" "}{help_hint(i18n, "stage2.field.encrypt_endpoint.help")}
                </ShellLine>
                {props.field_validation_block(SessionDraftField::WgPublicKey, html! {
                    <ShellLine>
                        <ShellPrompt>{i18n.t("stage2.field.wg_key")}</ShellPrompt>
                        {" "}
                        <ShellInput
                            value={draft.wg_public_key.clone()}
                            on_change={props.update_field_cb(SessionDraftField::WgPublicKey)}
                            class={props.input_class(SessionDraftField::WgPublicKey)}
                            frame_class={props.input_frame_class(SessionDraftField::WgPublicKey)}
                            on_focus={props.on_focus_cb(SessionDraftField::WgPublicKey)}
                            on_blur={props.on_blur_cb(SessionDraftField::WgPublicKey)}
                            placeholder={i18n.t("stage2.field.wg_key.placeholder")}
                            disabled={props.loading}
                        />
                    </ShellLine>
                })}
            </div>

            <div class="autopeer-form-section">
                {live_validation_block(i18n, lv.tunnel_message.as_deref(), html! {
                    <>
                        <span class="autopeer-section-label">{i18n.t("stage2.section.tunnel")}</span>
                        <p class="text-secondary">
                            {i18n.t("stage2.section.tunnel.help")}
                        </p>
                        {live_validation_block(i18n, lv.peer4_message.as_deref(), html! {
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.peer4")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.peer4.clone()}
                                    on_change={props.update_field_cb(SessionDraftField::Peer4)}
                                    class={props.input_class(SessionDraftField::Peer4)}
                                    frame_class={props.input_frame_class(SessionDraftField::Peer4)}
                                    on_focus={props.on_focus_cb(SessionDraftField::Peer4)}
                                    on_blur={props.on_blur_cb(SessionDraftField::Peer4)}
                                    placeholder={i18n.t("stage2.field.peer4.placeholder")}
                                    disabled={props.loading}
                                />
                            </ShellLine>
                        })}
                        if props.show_node_ipv4 {
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.own4_node")}</ShellPrompt>
                                {" "}
                                <span class="text-secondary">
                                    {props.node_inventory_ipv4.clone().unwrap_or_else(|| i18n.t("stage2.field.own4_node.no_inventory").to_string())}
                                </span>
                            </ShellLine>
                        }
                        {live_validation_block_multi(i18n, &lv.peer6_messages, html! {
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.peer6")}</ShellPrompt>
                                {" "}
                                <ShellInput
                                    value={draft.peer6.clone()}
                                    on_change={props.on_peer6_change.clone()}
                                    class={props.input_class(SessionDraftField::Peer6)}
                                    frame_class={props.input_frame_class(SessionDraftField::Peer6)}
                                    on_focus={props.on_focus_cb(SessionDraftField::Peer6)}
                                    on_blur={props.on_peer6_blur.clone()}
                                    placeholder={i18n.t("stage2.field.peer6.placeholder")}
                                    disabled={props.loading}
                                />
                            </ShellLine>
                        })}
                        if props.peer6_kind == Some(Peer6AddressKind::LinkLocal) {
                            {live_validation_block(i18n, lv.own6_message.as_deref(), html! {
                                <ShellLine>
                                    <ShellPrompt>{i18n.t("stage2.field.own6_link_local")}</ShellPrompt>
                                    {" "}
                                    <ShellInput
                                        value={draft.own6.clone()}
                                        on_change={props.update_field_cb(SessionDraftField::Own6)}
                                        class={props.input_class(SessionDraftField::Own6)}
                                        frame_class={props.input_frame_class(SessionDraftField::Own6)}
                                        on_focus={props.on_focus_cb(SessionDraftField::Own6)}
                                        on_blur={props.on_blur_cb(SessionDraftField::Own6)}
                                        placeholder={props.own6_placeholder.clone()}
                                        disabled={props.loading}
                                    />
                                </ShellLine>
                            })}
                        } else if props.peer6_kind == Some(Peer6AddressKind::Ula) {
                            <ShellLine>
                                <ShellPrompt>{i18n.t("stage2.field.own6_node")}</ShellPrompt>
                                {" "}
                                <span class="text-secondary">
                                    {props.node_inventory_ipv6.clone().unwrap_or_else(|| i18n.t("stage2.field.own6_node.no_inventory").to_string())}
                                </span>
                            </ShellLine>
                        }
                    </>
                })}
            </div>

            <div class="autopeer-form-section">
                <span class="autopeer-section-label">{i18n.t("stage2.section.families")}</span>
                <p class="text-secondary">
                    {i18n.t("stage2.section.families.help")}
                </p>
                {live_validation_block(i18n, lv.families_message.as_deref(), html! {
                    <ShellLine>
                        <ShellPrompt>{i18n.t("stage2.field.families")}</ShellPrompt>
                        {" "}
                        <span class="autopeer-toggle-row">
                            <span class={toggle_item_class(lv.highlight_ipv4)}>
                                <ShellToggle
                                    active={draft.ipv4}
                                    on_toggle={props.on_toggle_ipv4.clone()}
                                    label={i18n.t("stage2.field.families.ipv4_label")}
                                />
                            </span>
                            {" "}
                            <span class={toggle_item_class(lv.highlight_ipv6)}>
                                <ShellToggle
                                    active={draft.ipv6}
                                    on_toggle={props.on_toggle_ipv6.clone()}
                                    label={i18n.t("stage2.field.families.ipv6_label")}
                                />
                            </span>
                        </span>
                    </ShellLine>
                })}
            </div>

            <div class="autopeer-form-section">
                <span class="autopeer-section-label">{i18n.t("stage2.section.bgp")}</span>
                <p class="text-secondary">
                    {i18n.t("stage2.section.bgp.help")}
                </p>
                {live_validation_block(i18n, lv.bgp_message.as_deref(), html! {
                    <ShellLine>
                        <ShellPrompt>{i18n.t("stage2.field.bgp_features")}</ShellPrompt>
                        {" "}
                        <span class="autopeer-toggle-row">
                            <span class={toggle_item_class(lv.highlight_mp_bgp)}>
                                <ShellToggle
                                    active={draft.mp_bgp}
                                    on_toggle={props.on_toggle_mp_bgp.clone()}
                                    label={i18n.t("stage2.field.bgp.mpbgp_label")}
                                />
                            </span>
                            {" "}
                            <span class={toggle_item_class(lv.highlight_extended_next_hop)}>
                                <ShellToggle
                                    active={draft.extended_next_hop}
                                    on_toggle={props.on_toggle_extended_next_hop.clone()}
                                    label={i18n.t("stage2.field.bgp.enh_label")}
                                />
                            </span>
                        </span>
                    </ShellLine>
                })}
            </div>

            <div class="autopeer-form-section">
                <span class="autopeer-section-label">{i18n.t("stage2.section.policy")}</span>
                <p class="text-secondary">
                    {i18n.t(draft.peering_strategy.i18n_description_key())}
                </p>
                <ShellLine>
                    <ShellPrompt>{i18n.t("stage2.field.policy")}</ShellPrompt>
                    {" "}
                    <ShellSelect
                        value={draft.peering_strategy.as_str()}
                        on_change={props.on_change_peering_strategy.clone()}
                    >
                        {
                            for ALL_PEERING_STRATEGIES.iter().map(|strategy| html! {
                                <option value={strategy.as_str()}>{i18n.t(strategy.i18n_label_key())}</option>
                            })
                        }
                    </ShellSelect>
                </ShellLine>
            </div>

            <details class="autopeer-advanced">
                <summary>{i18n.t("stage2.advanced.summary")}</summary>
                <div class="autopeer-form-section autopeer-form-section--advanced">
                    if draft.mp_bgp {
                        <ShellLine>
                            <ShellPrompt>{i18n.t("stage2.field.bgp.transport")}</ShellPrompt>
                            {" "}
                            <ShellSelect
                                value={draft.selected_mp_bgp_transport().as_str()}
                                on_change={props.on_change_mp_bgp_transport.clone()}
                            >
                                {
                                    for ALL_MP_BGP_TRANSPORTS.iter().map(|transport| html! {
                                        <option value={transport.as_str()}>{humanize_ip_support(i18n, transport.as_str())}</option>
                                    })
                                }
                            </ShellSelect>
                        </ShellLine>
                    }
                    <ShellLine>
                        <ShellPrompt>{i18n.t("stage2.field.comment")}</ShellPrompt>
                        {" "}
                        <ShellInput
                            value={draft.comment.clone()}
                            on_change={props.on_comment_change.clone()}
                            placeholder={i18n.t("stage2.field.comment.placeholder")}
                            disabled={props.loading}
                        />
                    </ShellLine>
                    {props.field_validation_block(SessionDraftField::Keepalive, html! {
                        <ShellLine>
                            <ShellPrompt>{i18n.t("stage2.field.keepalive")}</ShellPrompt>
                            {" "}
                            <ShellInput
                                value={draft.keepalive.clone()}
                                on_change={props.update_field_cb(SessionDraftField::Keepalive)}
                                class={props.input_class(SessionDraftField::Keepalive)}
                                frame_class={props.input_frame_class(SessionDraftField::Keepalive)}
                                on_focus={props.on_focus_cb(SessionDraftField::Keepalive)}
                                on_blur={props.on_blur_cb(SessionDraftField::Keepalive)}
                                placeholder={i18n.t("stage2.field.keepalive.placeholder")}
                                disabled={props.loading}
                            />
                        </ShellLine>
                    })}
                    {props.field_validation_block(SessionDraftField::Mtu, html! {
                        <ShellLine>
                            <ShellPrompt>{i18n.t("stage2.field.mtu")}</ShellPrompt>
                            {" "}
                            <ShellInput
                                value={draft.mtu.clone()}
                                on_change={props.update_field_cb(SessionDraftField::Mtu)}
                                class={props.input_class(SessionDraftField::Mtu)}
                                frame_class={props.input_frame_class(SessionDraftField::Mtu)}
                                on_focus={props.on_focus_cb(SessionDraftField::Mtu)}
                                on_blur={props.on_blur_cb(SessionDraftField::Mtu)}
                                placeholder={i18n.t("stage2.field.mtu.placeholder")}
                                disabled={props.loading}
                            />
                        </ShellLine>
                    })}
                    {props.field_validation_block(SessionDraftField::Psk, html! {
                        <ShellLine>
                            <ShellPrompt>{i18n.t("stage2.field.psk")}</ShellPrompt>
                            {" "}
                            <ShellInput
                                value={draft.psk.clone()}
                                on_change={props.update_field_cb(SessionDraftField::Psk)}
                                class={props.input_class(SessionDraftField::Psk)}
                                frame_class={props.input_frame_class(SessionDraftField::Psk)}
                                on_focus={props.on_focus_cb(SessionDraftField::Psk)}
                                on_blur={props.on_blur_cb(SessionDraftField::Psk)}
                                placeholder={if draft.has_psk { i18n.t("stage2.field.psk.placeholder.existing") } else { i18n.t("stage2.field.psk.placeholder") }}
                                disabled={props.loading}
                            />
                            {" "}
                            <ShellButton
                                text={if props.psk_copied {
                                    i18n.t("stage2.field.psk.copied")
                                } else if draft.has_psk || !draft.psk.is_empty() {
                                    i18n.t("stage2.field.psk.clear")
                                } else {
                                    i18n.t("stage2.field.psk.generate")
                                }}
                                onclick={props.on_psk_action.clone()}
                                disabled={props.loading || props.psk_copied}
                            />
                            {" "}{help_hint(i18n, "stage2.field.psk.help")}
                        </ShellLine>
                    })}
                </div>
            </details>

            {render_ongoing_tasks(i18n, &props.ongoing_tasks)}
            {render_error(i18n, &props.error)}

            <div class="autopeer-inline-actions">
                if props.editing_node.is_some() {
                    <ShellButton text={i18n.t("action.cancel_edit")} onclick={props.on_cancel_edit.clone()} disabled={props.loading} />
                    <ShellButton
                        text={if props.retire_confirmation { i18n.t("action.confirm_retirement") } else { i18n.t("action.retire_session") }}
                        onclick={props.on_retire_selected_session.clone()}
                        disabled={props.loading}
                    />
                    <ShellButton
                        text={if props.delete_confirmation { i18n.t("action.confirm_deletion") } else { i18n.t("action.delete_session") }}
                        onclick={props.on_delete_selected_session.clone()}
                        disabled={props.loading}
                    />
                } else {
                    <ShellButton text={i18n.t("action.back_to_nodes")} onclick={props.on_change_node.clone()} disabled={props.loading} />
                }
                <ShellButton
                    text={if props.editing_node.is_some() { i18n.t("action.review_your_update") } else { i18n.t("action.review_your_change") }}
                    onclick={props.on_continue_to_review.clone()}
                    disabled={
                        props.loading
                            || (props.editing_node.is_none() && draft.node.trim().is_empty())
                            || !props.draft_is_valid
                    }
                />
            </div>
        </article>
    }
}
