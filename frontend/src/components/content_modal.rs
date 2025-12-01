use super::shell::ShellLine;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent, window};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContentModalProps {
    pub visible: bool,
    pub content: AttrValue,
    pub command: Option<AttrValue>,
    pub on_close: Callback<()>,
}

#[function_component(ContentModal)]
pub fn content_modal(props: &ContentModalProps) -> Html {
    {
        let visible = props.visible;
        let on_close = props.on_close.clone();
        use_effect_with(visible, move |visible| {
            let mut cleanup_listener: Option<Box<dyn FnOnce()>> = None;

            if let Some(window) = window()
                && let Some(body) = window.document().and_then(|d| d.body())
            {
                if *visible {
                    let _ = body.set_attribute("content-modal-locked", "true");
                    if let Some(active) = window.document().and_then(|d| d.active_element())
                        && let Ok(html_el) = active.dyn_into::<HtmlElement>()
                    {
                        let _ = html_el.blur();
                    }

                    let on_close = on_close.clone();
                    let listener = Closure::<dyn Fn(KeyboardEvent)>::wrap(Box::new(
                        move |e: KeyboardEvent| {
                            if e.key() == "Escape" {
                                on_close.emit(());
                            }
                        },
                    ));

                    let _ = window.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );

                    cleanup_listener = Some(Box::new(move || {
                        let _ = window.remove_event_listener_with_callback(
                            "keydown",
                            listener.as_ref().unchecked_ref(),
                        );
                        drop(listener);
                    }));
                } else {
                    let _ = body.remove_attribute("content-modal-locked");
                }
            }

            move || {
                if let Some(body) = window().and_then(|w| w.document()).and_then(|d| d.body()) {
                    let _ = body.remove_attribute("content-modal-locked");
                }
                if let Some(cleanup) = cleanup_listener {
                    cleanup();
                }
            }
        });
    }

    if !props.visible {
        return html! {};
    }

    let on_close = props.on_close.clone();
    let stop_click = Callback::from(|e: MouseEvent| e.stop_propagation());

    let (prompt, cmd_text) = if let Some(full_cmd) = &props.command {
        if let Some(idx) = full_cmd.find("$ ") {
            let (p, c) = full_cmd.split_at(idx + 2);
            (
                Some(AttrValue::from(p.to_string())),
                Some(AttrValue::from(c.to_string())),
            )
        } else {
            (None, Some(full_cmd.clone()))
        }
    } else {
        (None, None)
    };

    html! {
        <div
            class="modal-backdrop"
            onclick={move |_| on_close.emit(())}
            tabindex="0"
        >
            <div
                class="modal-content"
                onclick={stop_click.clone()}
            >
                {
                    if props.command.is_some() {
                        html! {
                            <ShellLine
                                prompt={prompt}
                                command={cmd_text}
                                style="margin-bottom: 0; border-bottom: none; border-radius: 4px 4px 0 0;"
                            />
                        }
                    } else {
                        html! {}
                    }
                }
                <pre
                    style={
                        if props.command.is_some() {
                            "margin: 0; border: 1px solid var(--border); border-top: none; border-radius: 0 0 4px 4px;"
                        } else {
                            ""
                        }
                    }
                >
                    { &props.content }
                </pre>
            </div>
        </div>
    }
}
