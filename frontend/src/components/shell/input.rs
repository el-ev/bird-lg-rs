use crate::utils::sleep_ms;
use wasm_bindgen_futures::spawn_local;
use web_sys::{FocusEvent, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellInputProps {
    pub value: String,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
}

// FIXME: Text selection is completely broken
#[function_component(ShellInput)]
pub fn shell_input(props: &ShellInputProps) -> Html {
    let selection_start = use_state(|| 0);
    let selection_end = use_state(|| 0);
    let is_focused = use_state(|| false);
    let input_ref = use_node_ref();

    {
        let len = props.value.chars().count();
        if *selection_start > len {
            selection_start.set(len);
        }
        if *selection_end > len {
            selection_end.set(len);
        }
    }

    let update_selection = {
        let selection_start = selection_start.clone();
        let selection_end = selection_end.clone();
        move |input: HtmlInputElement| {
            if let Ok(Some(start)) = input.selection_start() {
                selection_start.set(start as usize);
            }
            if let Ok(Some(end)) = input.selection_end() {
                selection_end.set(end as usize);
            }
        }
    };

    let on_input = {
        let on_change = props.on_change.clone();
        let update_selection = update_selection.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            on_change.emit(input.value());
            update_selection(input);
        })
    };

    let on_keydown = {
        let update_selection = update_selection.clone();
        Callback::from(move |e: KeyboardEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let update_selection = update_selection.clone();
            spawn_local(async move {
                sleep_ms(0).await;
                update_selection(input);
            });
        })
    };

    let on_keyup = {
        let update_selection = update_selection.clone();
        Callback::from(move |e: KeyboardEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            update_selection(input);
        })
    };

    let on_click = {
        let update_selection = update_selection.clone();
        Callback::from(move |e: MouseEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            update_selection(input);
        })
    };

    let on_focus = {
        let update_selection = update_selection.clone();
        let is_focused = is_focused.clone();
        Callback::from(move |e: FocusEvent| {
            is_focused.set(true);
            let input: HtmlInputElement = e.target_unchecked_into();
            let update_selection = update_selection.clone();
            spawn_local(async move {
                sleep_ms(0).await;

                if let (Ok(Some(start)), Ok(Some(end))) =
                    (input.selection_start(), input.selection_end())
                {
                    let value = input.value();
                    let js_len = value.encode_utf16().count() as u32;

                    if start == 0 && end == js_len && js_len > 0 {
                        let _ = input.set_selection_range(js_len, js_len);
                        update_selection(input);
                    }
                }
            });
        })
    };

    let on_blur = {
        let is_focused = is_focused.clone();
        Callback::from(move |_: FocusEvent| {
            is_focused.set(false);
        })
    };

    let on_wrapper_click = {
        let input_ref = input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        })
    };

    html! {
        <div class="shell-input-wrapper" onclick={on_wrapper_click}>
            {
                if props.value.is_empty() {
                    html! {
                        <>
                            <span class="shell-input-content placeholder">{ &props.placeholder }</span>
                            <span class="shell-cursor" style="position: absolute; left: 0;">{"_"}</span>
                        </>
                    }
                } else {
                    let chars: Vec<char> = props.value.chars().collect();
                    let start = *selection_start;
                    let end = *selection_end;

                    if start != end && *is_focused {
                        // Selection mode
                        let min = start.min(end);
                        let max = start.max(end);
                        let left: String = chars[0..min].iter().collect();
                        let selected: String = chars[min..max].iter().collect();
                        let right: String = chars[max..].iter().collect();
                        html! {
                            <span class="shell-input-content">
                                { left }
                                <span class="shell-input-selection">{ selected }</span>
                                { right }
                            </span>
                        }
                    } else {
                        // Cursor mode
                        let pos = start;
                        if pos >= chars.len() {
                            html! {
                                <span class="shell-input-content">
                                    { &props.value }
                                    <span class="shell-cursor">{"_"}</span>
                                </span>
                            }
                        } else {
                            let left: String = chars[0..pos].iter().collect();
                            let current = chars[pos];
                            let right: String = chars[pos+1..].iter().collect();
                            html! {
                                <span class="shell-input-content">
                                    { left }
                                    <span class="shell-cursor-char">{ current }</span>
                                    { right }
                                </span>
                            }
                        }
                    }
                }
            }
            <input
                ref={input_ref}
                class="shell-input-hidden"
                type="text"
                value={props.value.clone()}
                oninput={on_input}
                onkeydown={on_keydown}
                onkeyup={on_keyup}
                onclick={on_click}
                onfocus={on_focus}
                onblur={on_blur}
                disabled={props.disabled}
            />
        </div>
    }
}
