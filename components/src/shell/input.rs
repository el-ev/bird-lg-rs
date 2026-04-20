use unicode_width::UnicodeWidthStr;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShellInputProps {
    pub value: AttrValue,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub frame_class: Classes,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub multiline: bool,
    #[prop_or(4)]
    pub rows: usize,
    #[prop_or_default]
    pub on_focus: Callback<FocusEvent>,
    #[prop_or_default]
    pub on_blur: Callback<FocusEvent>,
    #[prop_or_default]
    pub on_keydown: Callback<KeyboardEvent>,
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
        (!props.multiline).then_some("shell-input-frame--inline"),
        props.frame_class.clone()
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
                    onfocus={props.on_focus.clone()}
                    onblur={props.on_blur.clone()}
                    onkeydown={props.on_keydown.clone()}
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
            let on_focus = props.on_focus.clone();
            Callback::from(move |event: FocusEvent| {
                if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                    cursor_col.set(read_cursor_position(&input));
                }
                on_focus.emit(event);
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
                    onblur={props.on_blur.clone()}
                    onkeydown={props.on_keydown.clone()}
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
    let value_width = display_width(props.value.as_str());
    let placeholder_width = display_width(props.placeholder.as_str());
    if value_width > 0 {
        value_width
    } else {
        placeholder_width.max(1)
    }
}

fn read_cursor_position(input: &HtmlInputElement) -> usize {
    input
        .selection_start()
        .ok()
        .flatten()
        .map(|pos| cursor_col_for_utf16_position(&input.value(), pos as usize))
        .unwrap_or_else(|| cursor_col_for_value(&input.value()))
}

fn cursor_col_for_value(value: &str) -> usize {
    display_width(value)
}

fn cursor_col_for_utf16_position(value: &str, utf16_pos: usize) -> usize {
    let byte_pos = byte_index_for_utf16_position(value, utf16_pos);
    display_width(&value[..byte_pos])
}

fn byte_index_for_utf16_position(value: &str, utf16_pos: usize) -> usize {
    let mut consumed_utf16 = 0;

    for (byte_pos, ch) in value.char_indices() {
        let next_utf16 = consumed_utf16 + ch.len_utf16();
        if utf16_pos < next_utf16 {
            return byte_pos;
        }
        if utf16_pos == next_utf16 {
            return byte_pos + ch.len_utf8();
        }
        consumed_utf16 = next_utf16;
    }

    value.len()
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

#[cfg(test)]
mod tests {
    use yew::{AttrValue, Callback, Classes};

    use super::{
        ShellInputProps, cursor_col_for_utf16_position, cursor_col_for_value, inline_input_width,
    };

    fn build_props(value: &str, placeholder: &str) -> ShellInputProps {
        ShellInputProps {
            value: AttrValue::from(value.to_owned()),
            on_change: Callback::noop(),
            class: Classes::new(),
            frame_class: Classes::new(),
            placeholder: AttrValue::from(placeholder.to_owned()),
            disabled: false,
            multiline: false,
            rows: 4,
            on_focus: Callback::noop(),
            on_blur: Callback::noop(),
            on_keydown: Callback::noop(),
        }
    }

    #[test]
    fn inline_input_width_matches_placeholder_length_when_empty() {
        let props = build_props("", "424242xxxx");
        assert_eq!(inline_input_width(&props), 10);
    }

    #[test]
    fn inline_input_width_uses_display_width_for_wide_placeholders() {
        let props = build_props("", "中文");
        assert_eq!(inline_input_width(&props), 4);
    }

    #[test]
    fn inline_input_width_tracks_value_even_when_placeholder_is_longer() {
        let props = build_props("20454", "the default port, e.g. 20454");
        assert_eq!(inline_input_width(&props), 5);
    }

    #[test]
    fn inline_input_width_keeps_empty_inputs_clickable() {
        let props = build_props("", "");
        assert_eq!(inline_input_width(&props), 1);
    }

    #[test]
    fn cursor_col_counts_display_columns() {
        assert_eq!(cursor_col_for_value("dn42"), 4);
        assert_eq!(cursor_col_for_value("fd00::1"), 7);
        assert_eq!(cursor_col_for_value("中文"), 4);
        assert_eq!(cursor_col_for_value("a中"), 3);
    }

    #[test]
    fn cursor_col_maps_utf16_positions_to_display_columns() {
        assert_eq!(cursor_col_for_utf16_position("a中b", 0), 0);
        assert_eq!(cursor_col_for_utf16_position("a中b", 1), 1);
        assert_eq!(cursor_col_for_utf16_position("a中b", 2), 3);
        assert_eq!(cursor_col_for_utf16_position("a中b", 3), 4);
    }
}
