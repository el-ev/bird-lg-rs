use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellInputProps {
    pub value: AttrValue,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
}

#[function_component(ShellInput)]
pub fn shell_input(props: &ShellInputProps) -> Html {
    let input_ref = use_node_ref();
    let is_focused = use_state(|| false);
    let cursor_pos = use_state(|| 0);

    let update_cursor = {
        let cursor_pos = cursor_pos.clone();
        Callback::from(move |input: HtmlInputElement| {
            let pos = input.selection_start().unwrap_or(Some(0)).unwrap_or(0) as usize;
            cursor_pos.set(pos);
        })
    };

    let oninput = {
        let on_change = props.on_change.clone();
        let update_cursor = update_cursor.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            on_change.emit(input.value());
            update_cursor.emit(input);
        })
    };

    let onkeyup = {
        let update_cursor = update_cursor.clone();
        Callback::from(move |e: KeyboardEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            update_cursor.emit(input);
        })
    };

    let onfocus = {
        let is_focused = is_focused.clone();
        Callback::from(move |_| is_focused.set(true))
    };

    let onblur = {
        let is_focused = is_focused.clone();
        Callback::from(move |_| is_focused.set(false))
    };

    let on_wrapper_click = {
        let input_ref = input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        })
    };

    let render_content = {
        let value = props.value.as_str();
        let pos = *cursor_pos;
        let is_focused = *is_focused;

        if value.is_empty() {
            let placeholder = props.placeholder.as_str();
            if placeholder.is_empty() {
                html! {
                    if is_focused {
                        <span class="shell-cursor-char">{" "}</span>
                    }
                }
            } else {
                let mut chars = placeholder.chars();
                let first = chars.next().unwrap_or(' ');
                let rest: String = chars.collect();

                html! {
                    <span class="shell-input-content placeholder">
                        if is_focused {
                            <span class="shell-cursor-char">{ first }</span>
                        } else {
                            { first }
                        }
                        { rest }
                    </span>
                }
            }
        } else {
            let chars: Vec<char> = value.chars().collect();
            let len = chars.len();

            let pos = pos.min(len);

            if !is_focused {
                html! { <span class="shell-input-content">{ value }</span> }
            } else if pos >= len {
                html! {
                    <span class="shell-input-content">
                        { value }
                        // FIXME
                        <span class="shell-cursor-char" style="background-color: var(--bg-shell)">{" "}</span>
                    </span>
                }
            } else {
                let left: String = chars[0..pos].iter().collect();
                let current = chars[pos];
                let right: String = chars[pos + 1..].iter().collect();
                html! {
                    <span class="shell-input-content">
                        { left }
                        <span class="shell-cursor-char">{ current }</span>
                        { right }
                    </span>
                }
            }
        }
    };

    html! {
        <div class="shell-input-wrapper" onclick={on_wrapper_click}>
            { render_content }
            <input
                ref={input_ref}
                class="shell-input-hidden"
                type="text"
                value={&props.value}
                oninput={oninput}
                onkeyup={onkeyup}
                onfocus={onfocus}
                onblur={onblur}
                disabled={props.disabled}
            />
        </div>
    }
}
