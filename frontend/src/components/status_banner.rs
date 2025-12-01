use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusBannerProps {
    pub error: Option<AttrValue>,
    pub waiting_for_data: bool,
}

#[function_component(StatusBanner)]
pub fn status_banner(props: &StatusBannerProps) -> Html {
    if let Some(e) = &props.error {
        return html! {
            <div class="status-message status-message--error">
                 { e }
            </div>
        };
    }

    if props.waiting_for_data {
        return html! {
            <div class="status-message">
                { "Waiting for data..." }
            </div>
        };
    }

    html! {}
}
