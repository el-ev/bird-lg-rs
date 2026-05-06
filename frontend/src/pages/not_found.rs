use yew::prelude::*;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="not-found">
            <h3>{"404 — Page Not Found"}</h3>
            <p>{"The page you're looking for doesn't exist."}</p>
        </div>
    }
}
