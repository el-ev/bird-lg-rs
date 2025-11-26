use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusBannerProps {
    pub fetch_error: Option<String>,
    pub waiting_for_data: bool,
}

#[function_component(StatusBanner)]
pub fn status_banner(props: &StatusBannerProps) -> Html {
    if let Some(e) = &props.fetch_error {
        return html! {
            <div class="status-message status-message--error">
                 { e.to_owned() }
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
