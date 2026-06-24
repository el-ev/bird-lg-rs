use web_sys::{HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellLineProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub extra_class: Option<&'static str>,
}

#[function_component(ShellLine)]
pub fn shell_line(props: &ShellLineProps) -> Html {
    let class = if let Some(extra) = props.extra_class {
        format!("shell-line {extra}")
    } else {
        "shell-line".to_string()
    };
    html! {
        <div class={class}>{for props.children.iter()}</div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ShellPromptProps {
    pub text: AttrValue,
}

#[function_component(ShellPrompt)]
pub fn shell_prompt(props: &ShellPromptProps) -> Html {
    html! {
        <span class="shell-prompt">{&props.text}</span>
    }
}

#[derive(Properties, PartialEq)]
pub struct ShellTextAreaProps {
    pub value: String,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or(8u32)]
    pub rows: u32,
    #[prop_or_default]
    pub readonly: bool,
}

#[function_component(ShellTextArea)]
pub fn shell_textarea(props: &ShellTextAreaProps) -> Html {
    let on_change = props.on_change.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        let ta: HtmlTextAreaElement = e.target_unchecked_into();
        on_change.emit(ta.value());
    });

    html! {
        <textarea
            class="shell-textarea"
            rows={props.rows.to_string()}
            value={props.value.clone()}
            oninput={oninput}
            placeholder={props.placeholder.clone().unwrap_or_default()}
            disabled={props.disabled}
            readonly={props.readonly}
        />
    }
}

#[derive(Properties, PartialEq)]
pub struct ShellSelectProps {
    pub value: String,
    pub on_change: Callback<String>,
    pub options: Vec<String>,
    #[prop_or_default]
    pub disabled: bool,
}

#[function_component(ShellSelect)]
pub fn shell_select(props: &ShellSelectProps) -> Html {
    let on_change = props.on_change.clone();
    let onchange = Callback::from(move |e: Event| {
        let select: HtmlSelectElement = e.target_unchecked_into();
        on_change.emit(select.value());
    });

    html! {
        <select class="shell-select" onchange={onchange} disabled={props.disabled}>
            {for props.options.iter().map(|opt| {
                let selected = *opt == props.value;
                html! {
                    <option value={opt.clone()} selected={selected}>{opt.clone()}</option>
                }
            })}
        </select>
    }
}

#[derive(Properties, PartialEq)]
pub struct ShellButtonProps {
    pub text: AttrValue,
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub submit_style: bool,
}

#[function_component(ShellButton)]
pub fn shell_button(props: &ShellButtonProps) -> Html {
    let class = if props.submit_style {
        "shell-button shell-button--submit"
    } else {
        "shell-button"
    };
    html! {
        <button class={class} onclick={props.onclick.clone()} disabled={props.disabled}>
            {&props.text}
        </button>
    }
}
