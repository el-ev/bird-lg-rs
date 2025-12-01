use yew::{Callback, Children, Html, Properties, function_component, html, use_context};

use crate::{
    components::{content_modal::ContentModal, header::Header, status_banner::StatusBanner},
    store::{Action, AppStateHandle, modal::ModalAction},
};

#[derive(Properties, PartialEq)]
pub struct MainViewProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(MainView)]
pub fn main_view(props: &MainViewProps) -> Html {
    let state = use_context::<AppStateHandle>().expect("no app state found");
    let waiting_for_data = state.nodes.is_empty() && !state.data_ready;
    html! {
        <main class="hero">
            <div class="container">
                <Header
                    network_info={state.network_info.clone()}
                />

                <StatusBanner
                    error={state.error.clone()}
                    waiting_for_data={waiting_for_data}
                />
                <ContentModal
                    visible={state.modal.active}
                    content={state.modal.content.clone()}
                    command={state.modal.command.clone()}
                    on_close={
                        let state = state.clone();
                        Callback::from(move |_| {
                            state.dispatch(Action::Modal(ModalAction::Close));
                        })
                    }
                />
                {
                    if state.config_ready {
                        html! { { for props.children.iter() } }
                    } else {
                        html!{}
                    }
                }
            </div>
        </main>
    }
}
