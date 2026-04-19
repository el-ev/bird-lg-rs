use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellSelectProps {
    pub value: AttrValue,
    pub on_change: Callback<Event>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub options: Option<Vec<AttrValue>>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellSelect)]
pub fn shell_select(props: &ShellSelectProps) -> Html {
    let select_ref = use_node_ref();

    {
        let select_ref = select_ref.clone();
        let value = props.value.clone();
        use_effect_with(value, move |value| {
            if let Some(el) = select_ref.cast::<HtmlSelectElement>() {
                if el.value() != value.as_str() {
                    el.set_value(value);
                }
            }
            || ()
        });
    }

    html! {
        <select
            ref={select_ref}
            class={classes!("shell-select", props.class.clone())}
            onchange={&props.on_change}
        >
            {
                if let Some(options) = &props.options {
                    html! {
                        { for options.iter().map(|opt| html! {
                            <option value={opt} selected={opt == &props.value}>{ opt }</option>
                        }) }
                    }
                } else {
                    html! { { for props.children.iter() } }
                }
            }
        </select>
    }
}
