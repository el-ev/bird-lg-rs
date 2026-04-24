use yew::prelude::*;

use crate::{
    controller::OngoingTask,
    i18n::I18n,
    models::{NodeView, UiMessage},
    store::SessionDraft,
};
use ui_components::shell::ShellButton;

use super::{
    mp_bgp_transport_label, node_review_line, optional_review_item, render_error,
    render_inventory_peering_review, render_ongoing_tasks, review_item,
};

#[derive(Properties, PartialEq)]
pub struct ReviewProps {
    pub i18n: I18n,
    pub loading: bool,
    pub draft: SessionDraft,
    pub editing_node: Option<String>,
    pub selected_node: Option<NodeView>,
    pub active_asn: String,
    pub draft_is_valid: bool,
    pub ongoing_tasks: Vec<OngoingTask>,
    pub error: Option<UiMessage>,
    pub on_back_to_details: Callback<MouseEvent>,
    pub on_cancel_edit: Callback<MouseEvent>,
    pub on_change_node: Callback<MouseEvent>,
    pub on_submit_session: Callback<MouseEvent>,
}

#[function_component(ReviewPanel)]
pub fn review_panel(props: &ReviewProps) -> Html {
    let i18n = &props.i18n;
    let draft = &props.draft;

    html! {
        <article class="peering-card autopeer-panel">
            <div class="autopeer-panel-header">
                <p class="autopeer-panel-kicker">{i18n.t("stage3.kicker")}</p>
                <h3 class="autopeer-panel-title">{i18n.t("stage3.title")}</h3>
            </div>

            <div class="autopeer-review-grid">
                {review_item(
                    i18n.t("stage3.review.our_node"),
                    props.selected_node
                        .as_ref()
                        .map(|node| node_review_line(i18n, node))
                        .unwrap_or_else(|| i18n.t("stage3.review.not_selected").to_string()),
                )}
                {review_item(
                    i18n.t("stage3.review.endpoint"),
                    if draft.encrypt_endpoint {
                        format!("{} ({})", draft.endpoint, i18n.t("stage3.review.encrypt_endpoint.enabled"))
                    } else {
                        draft.endpoint.clone()
                    },
                )}
                {review_item(i18n.t("stage3.review.wg_key"), draft.wg_public_key.clone())}
                {review_item(
                    i18n.t("stage3.review.route_families"),
                    i18n.t(draft.families_label_key()).to_string(),
                )}
                {review_item(
                    i18n.t("stage3.review.bgp_behavior"),
                    format!(
                        "{}{}{}",
                        if draft.mp_bgp { i18n.t("stage3.review.bgp.mpbgp") } else { i18n.t("stage3.review.bgp.separate") },
                        if draft.mp_bgp {
                            format!(
                                " ({})",
                                mp_bgp_transport_label(i18n, draft.selected_mp_bgp_transport())
                            )
                        } else {
                            String::new()
                        },
                        if draft.extended_next_hop { i18n.t("stage3.review.bgp.enh_suffix") } else { "" },
                    ),
                )}
                {review_item(
                    i18n.t("stage3.review.routing_policy"),
                    i18n.t(draft.peering_strategy.i18n_label_key()).to_string(),
                )}
                {optional_review_item(i18n.t("stage3.review.peer4"), &draft.peer4)}
                {optional_review_item(i18n.t("stage3.review.peer6"), &draft.peer6)}
                {optional_review_item(i18n.t("stage3.review.own6"), &draft.own6)}
                {optional_review_item(i18n.t("stage3.review.keepalive"), &draft.keepalive)}
                {optional_review_item(i18n.t("stage3.review.mtu"), &draft.mtu)}
                {review_item(
                    i18n.t("stage3.review.psk"),
                    if !draft.psk.trim().is_empty() {
                        i18n.t("stage3.review.psk.set").to_string()
                    } else if draft.clear_psk {
                        i18n.t("stage3.review.psk.cleared").to_string()
                    } else if draft.has_psk {
                        i18n.t("stage3.review.psk.unchanged").to_string()
                    } else {
                        i18n.t("stage3.review.psk.not_set").to_string()
                    },
                )}

                {optional_review_item(i18n.t("stage3.review.note"), &draft.comment)}
            </div>

            {render_inventory_peering_review(i18n, props.selected_node.as_ref(), &props.active_asn)}

            {render_ongoing_tasks(i18n, &props.ongoing_tasks)}
            {render_error(i18n, &props.error)}

            <div class="autopeer-inline-actions">
                <ShellButton text={i18n.t("action.back_to_details")} onclick={props.on_back_to_details.clone()} disabled={props.loading} />
                if props.editing_node.is_some() {
                    <ShellButton text={i18n.t("action.cancel_edit")} onclick={props.on_cancel_edit.clone()} disabled={props.loading} />
                } else {
                    <ShellButton text={i18n.t("action.choose_another_node")} onclick={props.on_change_node.clone()} disabled={props.loading} />
                }
                <ShellButton
                    text={if props.editing_node.is_some() { i18n.t("action.open_update_pr") } else { i18n.t("action.open_create_pr") }}
                    onclick={props.on_submit_session.clone()}
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
