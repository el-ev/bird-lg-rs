use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellFormProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub onsubmit: Callback<SubmitEvent>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellForm)]
pub fn shell_form(props: &ShellFormProps) -> Html {
    html! {
        <form
            class={classes!("shell-line", "shell-form", props.class.clone())}
            onsubmit={props.onsubmit.clone()}
        >
            { for props.children.iter() }
        </form>
    }
}
