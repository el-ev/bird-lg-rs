use web_sys::{MouseEvent, window};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContentModalProps {
    pub visible: bool,
    pub content: String,
    pub on_close: Callback<()>,
}

#[function_component(ContentModal)]
pub fn content_modal(props: &ContentModalProps) -> Html {
    {
        let visible = props.visible;
        use_effect_with(visible, move |visible| {
            if let Some(body) = window().and_then(|w| w.document()).and_then(|d| d.body()) {
                if *visible {
                    let _ = body.set_attribute("content-modal-locked", "true");
                } else {
                    let _ = body.remove_attribute("content-modal-locked");
                }
            }

            move || {
                if let Some(body) = window().and_then(|w| w.document()).and_then(|d| d.body()) {
                    let _ = body.remove_attribute("content-modal-locked");
                }
            }
        });
    }

    if !props.visible {
        return html! {};
    }

    let on_close = props.on_close.clone();
    let stop_click = Callback::from(|e: MouseEvent| e.stop_propagation());

    html! {
        <div class="modal-backdrop" onclick={move |_| on_close.emit(())}>
            <div class="modal-content" onclick={stop_click.clone()}>
                <pre>{ &props.content }</pre>
            </div>
        </div>
    }
}
