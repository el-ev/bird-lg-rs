use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellToggleProps {
    pub active: bool,
    pub on_toggle: Callback<()>,
    #[prop_or_default]
    pub label: Option<String>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellToggle)]
pub fn shell_toggle(props: &ShellToggleProps) -> Html {
    let onclick = props.on_toggle.reform(|_: MouseEvent| ());

    let onkeydown = {
        let cb = props.on_toggle.clone();
        Callback::from(move |e: KeyboardEvent| {
            if matches!(e.key().as_str(), "Enter" | " ") {
                e.prevent_default();
                cb.emit(());
            }
        })
    };

    html! {
        <span
            class={classes!("shell-toggle", if props.active { "active" } else { "" })}
            onclick={onclick}
            tabindex="0"
            onkeydown={onkeydown}
        >
            {
                if let Some(label) = &props.label {
                    html! { { label } }
                } else {
                    html! { { for props.children.iter() } }
                }
            }
        </span>
    }
}
