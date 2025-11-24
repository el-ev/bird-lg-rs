use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellPromptProps {
    pub children: Children,
}

#[function_component(ShellPrompt)]
pub fn shell_prompt(props: &ShellPromptProps) -> Html {
    html! { <span class="shell-prompt">{ for props.children.iter() }</span> }
}
