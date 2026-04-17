use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellInputProps {
    pub value: AttrValue,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub multiline: bool,
    #[prop_or(4)]
    pub rows: usize,
}

#[function_component(ShellInput)]
pub fn shell_input(props: &ShellInputProps) -> Html {
    let input_ref = use_node_ref();
    let cursor_col = use_state(|| cursor_col_for_value(props.value.as_str()));

    {
        let cursor_col = cursor_col.clone();
        let value = props.value.clone();
        use_effect_with(value, move |value| {
            cursor_col.set(cursor_col_for_value(value.as_str()));
            || ()
        });
    }

    let frame_classes = classes!(
        "shell-input-frame",
        props.multiline.then_some("shell-input-frame--multiline"),
        (!props.multiline).then_some("shell-input-frame--inline")
    );
    let classes = classes!(
        "shell-input",
        (!props.multiline).then_some("shell-input--inline"),
        props.multiline.then_some("shell-input--multiline"),
        props.class.clone()
    );

    if props.multiline {
        let on_change = props.on_change.clone();
        let oninput = Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            on_change.emit(input.value());
        });

        html! {
            <div class={frame_classes}>
                <textarea
                    class={classes}
                    value={props.value.clone()}
                    placeholder={props.placeholder.clone()}
                    oninput={oninput}
                    rows={props.rows.to_string()}
                    disabled={props.disabled}
                    spellcheck="false"
                />
            </div>
        }
    } else {
        let update_cursor = {
            let cursor_col = cursor_col.clone();
            Callback::from(move |input: HtmlInputElement| {
                cursor_col.set(read_cursor_position(&input));
            })
        };

        let on_change = props.on_change.clone();
        let width_style = format!("--shell-input-ch: {}", inline_input_width(props));
        let oninput = {
            let update_cursor = update_cursor.clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                update_cursor.emit(input.clone());
                on_change.emit(input.value());
            })
        };
        let onkeyup = {
            let update_cursor = update_cursor.clone();
            Callback::from(move |e: KeyboardEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                update_cursor.emit(input);
            })
        };
        let onclick = {
            let update_cursor = update_cursor.clone();
            Callback::from(move |e: MouseEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                update_cursor.emit(input);
            })
        };
        let onmouseup = {
            let update_cursor = update_cursor.clone();
            Callback::from(move |e: MouseEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                update_cursor.emit(input);
            })
        };
        let onfocus = {
            let input_ref = input_ref.clone();
            let cursor_col = cursor_col.clone();
            Callback::from(move |_| {
                if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                    cursor_col.set(read_cursor_position(&input));
                }
            })
        };

        html! {
            <span
                class={frame_classes}
                style={format!("{width_style}; --shell-caret-col: {}", *cursor_col)}
            >
                <input
                    ref={input_ref}
                    class={classes}
                    type="text"
                    value={props.value.clone()}
                    placeholder={props.placeholder.clone()}
                    oninput={oninput}
                    onkeyup={onkeyup}
                    onclick={onclick}
                    onmouseup={onmouseup}
                    onfocus={onfocus}
                    disabled={props.disabled}
                    spellcheck="false"
                    autocomplete="off"
                    autocapitalize="off"
                    autocorrect="off"
                />
            </span>
        }
    }
}

fn inline_input_width(props: &ShellInputProps) -> usize {
    let value_width = props.value.chars().count();
    let placeholder_width = props.placeholder.chars().count();
    value_width.max(placeholder_width).max(8) + 1
}

fn read_cursor_position(input: &HtmlInputElement) -> usize {
    input
        .selection_start()
        .ok()
        .flatten()
        .map(|pos| pos as usize)
        .unwrap_or_else(|| cursor_col_for_value(&input.value()))
}

fn cursor_col_for_value(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
mod tests {
    use yew::{AttrValue, Callback, Classes};

    use super::{ShellInputProps, cursor_col_for_value, inline_input_width};

    fn build_props(value: &str, placeholder: &str) -> ShellInputProps {
        ShellInputProps {
            value: AttrValue::from(value.to_owned()),
            on_change: Callback::noop(),
            class: Classes::new(),
            placeholder: AttrValue::from(placeholder.to_owned()),
            disabled: false,
            multiline: false,
            rows: 4,
        }
    }

    #[test]
    fn inline_input_width_reserves_room_for_placeholder() {
        let props = build_props("", "424242xxxx");
        assert_eq!(inline_input_width(&props), 11);
    }

    #[test]
    fn inline_input_width_grows_with_value_length() {
        let props = build_props("birdc show route", "<target>");
        assert_eq!(inline_input_width(&props), 17);
    }

    #[test]
    fn cursor_col_counts_characters() {
        assert_eq!(cursor_col_for_value("dn42"), 4);
        assert_eq!(cursor_col_for_value("fd00::1"), 7);
    }
}
