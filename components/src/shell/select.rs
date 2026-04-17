use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellSelectProps {
    pub value: AttrValue,
    pub on_change: Callback<Event>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub options: Option<Vec<AttrValue>>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ShellSelect)]
pub fn shell_select(props: &ShellSelectProps) -> Html {
    html! {
        <select
            class={classes!("shell-select", props.class.clone())}
            value={&props.value}
            onchange={&props.on_change}
        >
            {
                if let Some(options) = &props.options {
                    html! {
                        { for options.iter().map(|opt| html! {
                            <option value={opt}>{ opt }</option>
                        }) }
                    }
                } else {
                    html! { { for props.children.iter() } }
                }
            }
        </select>
    }
}
