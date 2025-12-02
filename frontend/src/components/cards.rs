use common::models::PeeringInfo;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, window};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PeeringNodeCardProps {
    pub node_name: AttrValue,
    pub node_info: PeeringInfo,
}

#[function_component(PeeringNodeCard)]
pub fn peering_node_card(props: &PeeringNodeCardProps) -> Html {
    let has_wg = props.node_info.wg_pubkey.is_some();

    html! {
        <article class="peering-card peering-node">
            <div class="peering-node-header">
                <h4 class="peering-node-title">{&props.node_name}</h4>
                if let Some(comment) = &props.node_info.comment {
                    <span class="peering-node-meta">{comment}</span>
                }
            </div>
            <dl class="peering-grid">
                <PeeringField label="Endpoint" value={props.node_info.endpoint.clone()} />
                <PeeringField label="IPv4" value={props.node_info.ipv4.clone()} />
                <PeeringField label="IPv6" value={props.node_info.ipv6.clone()} />
                <PeeringField label="IPv6 Link-Local" value={props.node_info.link_local_ipv6.clone()} />
                if has_wg {
                    <PeeringField label="Tunnel" value={"WireGuard"} />
                }
                <PeeringField label="WG Public Key" value={props.node_info.wg_pubkey.clone()} />
            </dl>
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub struct ContactCardProps {
    pub contacts: Vec<(String, String)>,
}

#[function_component(ContactCard)]
pub fn contact_card(props: &ContactCardProps) -> Html {
    if props.contacts.is_empty() {
        return html! {};
    }

    html! {
        <>
            <article class="peering-card peering-contact">
                <div class="peering-card-header">
                    <h3 class="peering-card-title">{"Contact"}</h3>
                </div>
                <dl class="peering-grid peering-contact-grid">
                    { for props.contacts.iter().map(|(label, value)| {
                        html! {
                            <PeeringField
                                label={label.clone()}
                                value={Some(value.clone())}
                                label_class="peering-contact-label"
                                value_class="peering-contact-value"
                            >
                            </PeeringField>
                        }
                    }) }
                </dl>
            </article>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct PeeringFieldProps {
    pub label: AttrValue,
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub label_class: Option<AttrValue>,
    #[prop_or_default]
    pub value_class: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(PeeringField)]
pub fn peering_field(props: &PeeringFieldProps) -> Html {
    if let Some(value) = &props.value {
        let label_class = props
            .label_class
            .as_ref()
            .map(|c| format!("peering-label {}", c))
            .unwrap_or_else(|| "peering-label".to_string());

        let value_class = props
            .value_class
            .as_ref()
            .map(|c| format!("peering-value {}", c))
            .unwrap_or_else(|| "peering-value".to_string());

        html! {
            <>
                <dt class={label_class}>{&props.label}</dt>
                <dd class={value_class} onclick={select_text}>
                    if props.children.is_empty() {
                        {value}
                    } else {
                        {props.children.clone()}
                    }
                </dd>
            </>
        }
    } else {
        html! {}
    }
}

fn select_text(e: MouseEvent) {
    if let Some(target) = e.target()
        && let Ok(element) = target.dyn_into::<HtmlElement>()
        && let Some(window) = window()
        && let Ok(Some(selection)) = window.get_selection()
    {
        let _ = selection.remove_all_ranges();
        if let Some(document) = window.document()
            && let Ok(range) = document.create_range()
            && range.select_node_contents(&element).is_ok()
        {
            let _ = selection.add_range(&range);
        }
    }
}
