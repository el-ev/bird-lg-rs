mod auth;
mod components;
mod i18n;

use auth::AuthFlow;
use i18n::{I18nProvider, Locale, use_i18n};
use yew::prelude::*;

#[function_component(AppInner)]
fn app_inner() -> Html {
    let i18n = use_i18n();

    let on_locale_change = {
        let i18n = i18n.clone();
        Callback::from(move |event: Event| {
            let target: web_sys::HtmlSelectElement = event.target_unchecked_into();
            if let Some(locale) = Locale::from_code(&target.value()) {
                i18n.set_locale(locale);
            }
        })
    };

    html! {
        <div class="hero">
            <div class="container">
                <div class="autopeer-page-header">
                    <h1 class="title">
                        <span class="title-flex">
                            <span class="title-link">{ i18n.t("app.title") }</span>
                            <span class="title-footnote">
                                { i18n.t("app.subtitle") }
                            </span>
                        </span>
                    </h1>
                    <span class="autopeer-language-control">
                        <span class="autopeer-language-label">{ i18n.t("nav.language") }</span>
                        <select class="shell-select" onchange={on_locale_change}>
                            { for Locale::ALL.iter().map(|loc| {
                                let selected = *loc == i18n.locale();
                                html! {
                                    <option value={loc.code()} selected={selected}>
                                        { loc.label() }
                                    </option>
                                }
                            }) }
                        </select>
                    </span>
                </div>
                <div class="auth-page">
                    <AuthFlow />
                </div>
            </div>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <I18nProvider>
            <AppInner />
        </I18nProvider>
    }
}
