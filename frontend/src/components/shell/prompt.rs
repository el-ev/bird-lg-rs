use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellPromptProps {
    #[prop_or_default]
    pub prefix: Option<AttrValue>,
    #[prop_or_default]
    pub suffix: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellPrompt)]
pub fn shell_prompt(props: &ShellPromptProps) -> Html {
    html! {
        <span class="shell-prompt">
            { if let Some(prefix) = &props.prefix { html! { { prefix } } } else { html! {} } }
            { for props.children.iter() }
            { if let Some(suffix) = &props.suffix { html! { { suffix } } } else { html! {} } }
        </span>
    }
}
