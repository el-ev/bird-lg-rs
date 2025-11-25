use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellButtonProps {
    #[prop_or("button".to_string())]
    pub type_: String,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub text: Option<String>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellButton)]
pub fn shell_button(props: &ShellButtonProps) -> Html {
    html! {
        <button
            type={props.type_.clone()}
            class="shell-button"
            onclick={props.onclick.clone()}
            disabled={props.disabled}
        >
            {
                if let Some(text) = &props.text {
                    html! { { text } }
                } else {
                    html! { { for props.children.iter() } }
                }
            }
        </button>
    }
}
