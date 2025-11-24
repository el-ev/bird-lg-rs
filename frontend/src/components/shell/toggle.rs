use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellToggleProps {
    pub active: bool,
    pub on_toggle: Callback<()>,
    pub children: Children,
}

#[function_component(ShellToggle)]
pub fn shell_toggle(props: &ShellToggleProps) -> Html {
    let on_toggle = props.on_toggle.clone();
    let onclick = Callback::from(move |_| on_toggle.emit(()));

    let on_toggle_key = props.on_toggle.clone();
    let onkeydown = Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Enter" || e.key() == " " {
            e.prevent_default();
            on_toggle_key.emit(());
        }
    });

    html! {
        <span
            class={classes!("shell-toggle", if props.active { "active" } else { "" })}
            onclick={onclick}
            tabindex="0"
            onkeydown={onkeydown}
        >
            { for props.children.iter() }
        </span>
    }
}
