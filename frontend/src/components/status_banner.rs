use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusBannerProps {
    pub fetch_error: Option<String>,
    pub waiting_for_data: bool,
}

#[function_component(StatusBanner)]
pub fn status_banner(props: &StatusBannerProps) -> Html {
    if props.fetch_error.is_some() {
        return html! {
            <div class="status-message status-message--error">
                { "Error connecting to backend" }
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
