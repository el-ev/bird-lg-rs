use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellToggleProps {
    pub active: bool,
    pub on_toggle: Callback<()>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellToggle)]
pub fn shell_toggle(props: &ShellToggleProps) -> Html {
    let onclick = props.on_toggle.reform(|_: MouseEvent| ());

    html! {
        <button
            type="button"
            class={classes!("shell-toggle", if props.active { "active" } else { "" })}
            onclick={onclick}
            role="switch"
            aria-checked={if props.active { "true" } else { "false" }}
        >
            <span class="shell-toggle__copy">
                <span class="shell-toggle__label">
                    {
                        if let Some(label) = &props.label {
                            html! { { label } }
                        } else {
                            html! { { for props.children.iter() } }
                        }
                    }
                </span>
            </span>
            <span class="shell-toggle__switch" aria-hidden="true">
                <span class="shell-toggle__thumb" />
            </span>
        </button>
    }
}
