use yew::prelude::*;

use super::ShellPrompt;

#[derive(Properties, PartialEq)]
pub struct ShellLineProps {
    #[prop_or_default]
    pub prompt: Option<AttrValue>,
    #[prop_or_default]
    pub command: Option<AttrValue>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellLine)]
pub fn shell_line(props: &ShellLineProps) -> Html {
    html! {
        <div class="shell-line" style={&props.style}>
            {
                if !props.children.is_empty() {
                    html! { { for props.children.iter() } }
                } else if let (Some(prompt), Some(command)) = (&props.prompt, &props.command) {
                    html! {
                        <>
                            <ShellPrompt>{ prompt.clone() }</ShellPrompt>
                            { command.clone() }
                        </>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
