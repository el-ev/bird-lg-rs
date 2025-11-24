use super::ShellPrompt;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellLineProps {
    pub prompt: String,
    pub command: String,
    #[prop_or_default]
    pub style: Option<String>,
}

#[function_component(ShellLine)]
pub fn shell_line(props: &ShellLineProps) -> Html {
    html! {
        <div class="shell-line" style={props.style.clone()}>
            <ShellPrompt>{ html! { &props.prompt } }</ShellPrompt>
            { html! { &props.command } }
        </div>
    }
}
