use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellPlainToggleProps {
    pub active: bool,
    pub on_toggle: Callback<()>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellPlainToggle)]
pub fn shell_plain_toggle(props: &ShellPlainToggleProps) -> Html {
    let onclick = props.on_toggle.reform(|_: MouseEvent| ());

    html! {
        <button
            type="button"
            class={classes!("shell-text-toggle", props.active.then_some("active"))}
            onclick={onclick}
            role="switch"
            aria-checked={if props.active { "true" } else { "false" }}
        >
            {
                if let Some(label) = &props.label {
                    html! { { label } }
                } else {
                    html! { { for props.children.iter() } }
                }
            }
        </button>
    }
}
