use ui_components::shell::ShellButton;
use yew::prelude::*;

use super::{
    humanize_ip_support, node_context_line, optional_review_item, render_error,
    render_inventory_peering_review, render_ongoing_tasks, review_item,
};
use crate::{
    controller::OngoingTask,
    i18n::I18n,
    models::{NodeView, UiMessage},
    store::SessionDraft,
};

#[derive(Properties, PartialEq)]
pub struct ReviewProps {
    pub i18n: I18n,
    pub loading: bool,
    pub draft: SessionDraft,
    pub original_draft: Option<SessionDraft>,
    pub editing_node: Option<String>,
    pub selected_node: Option<NodeView>,
    pub active_asn: String,
    pub looking_glass_site_href: String,
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
    let orig = props.original_draft.as_ref();
    let changed = |f: fn(&SessionDraft, &SessionDraft) -> bool| orig.is_some_and(|o| f(o, draft));

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
                        .map(|node| {
                            let context = node_context_line(i18n, node);
                            if context.is_empty() {
                                node.name.clone()
                            } else {
                                format!("{} ({})", node.name, context)
                            }
                        })
                        .unwrap_or_else(|| i18n.t("stage3.review.not_selected").to_string()),
                    false,
                )}
                {review_item(
                    i18n.t("stage3.review.endpoint"),
                    if draft.encrypt_endpoint {
                        format!("{} ({})", draft.endpoint, i18n.t("stage3.review.encrypt_endpoint.enabled"))
                    } else {
                        draft.endpoint.clone()
                    },
                    changed(|o, d| o.endpoint != d.endpoint || o.encrypt_endpoint != d.encrypt_endpoint),
                )}
                {review_item(
                    i18n.t("stage3.review.wg_key"),
                    draft.wg_public_key.clone(),
                    changed(|o, d| o.wg_public_key != d.wg_public_key),
                )}
                {review_item(
                    i18n.t("stage3.review.route_families"),
                    i18n.t(draft.families_label_key()).to_string(),
                    changed(|o, d| o.ipv4 != d.ipv4 || o.ipv6 != d.ipv6),
                )}
                {review_item(
                    i18n.t("stage3.review.bgp_behavior"),
                    format!(
                        "{}{}{}",
                        if draft.mp_bgp { i18n.t("stage3.review.bgp.mpbgp") } else { i18n.t("stage3.review.bgp.separate") },
                        if draft.mp_bgp {
                            format!(
                                " ({})",
                                humanize_ip_support(i18n, draft.selected_mp_bgp_transport().as_str())
                            )
                        } else {
                            String::new()
                        },
                        if draft.extended_next_hop { i18n.t("stage3.review.bgp.enh_suffix") } else { "" },
                    ),
                    changed(|o, d| o.mp_bgp != d.mp_bgp
                        || o.extended_next_hop != d.extended_next_hop
                        || o.mp_bgp_transport != d.mp_bgp_transport),
                )}
                {review_item(
                    i18n.t("stage3.review.routing_policy"),
                    i18n.t(draft.peering_strategy.i18n_label_key()).to_string(),
                    changed(|o, d| o.peering_strategy != d.peering_strategy),
                )}
                {optional_review_item(i18n.t("stage3.review.peer4"), &draft.peer4, changed(|o, d| o.peer4 != d.peer4))}
                {optional_review_item(i18n.t("stage3.review.peer6"), &draft.peer6, changed(|o, d| o.peer6 != d.peer6))}
                {optional_review_item(i18n.t("stage3.review.own6"), &draft.own6, changed(|o, d| o.own6 != d.own6))}
                {optional_review_item(i18n.t("stage3.review.keepalive"), &draft.keepalive, changed(|o, d| o.keepalive != d.keepalive))}
                {optional_review_item(i18n.t("stage3.review.mtu"), &draft.mtu, changed(|o, d| o.mtu != d.mtu))}
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
                    changed(|o, d| !d.psk.trim().is_empty() || d.clear_psk != o.clear_psk),
                )}

                {optional_review_item(i18n.t("stage3.review.note"), &draft.comment, changed(|o, d| o.comment != d.comment))}
            </div>

            {render_inventory_peering_review(i18n, props.selected_node.as_ref(), &props.active_asn, draft, &props.looking_glass_site_href)}

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
