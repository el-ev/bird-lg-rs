use yew::prelude::*;

use crate::i18n::I18nProvider;
use crate::page::AutoPeerPage;

#[function_component(AutoPeerApp)]
pub fn auto_peer_app() -> Html {
    html! {
        <I18nProvider>
            <AutoPeerPage />
        </I18nProvider>
    }
}
