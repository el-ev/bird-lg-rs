use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellSelectProps {
    pub value: String,
    pub on_change: Callback<Event>,
    #[prop_or_default]
    pub class: Classes,
    pub children: Children,
}

#[function_component(ShellSelect)]
pub fn shell_select(props: &ShellSelectProps) -> Html {
    html! {
        <select
            class={classes!("shell-select", props.class.clone())}
            value={props.value.clone()}
            onchange={props.on_change.clone()}
        >
            { for props.children.iter() }
        </select>
    }
}
