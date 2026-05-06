use yew::prelude::*;

use crate::{
    components::{content_modal::ContentModal, header::Header, status_banner::StatusBanner},
    store::{AppEvent, LgStateHandle, route_info::RouteInfoHandle},
    utils::clear_hash_route,
};

#[derive(Properties, PartialEq)]
pub struct MainViewProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(MainView)]
pub fn main_view(props: &MainViewProps) -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route_info = use_context::<RouteInfoHandle>().expect("no route info found");
    let waiting_for_data = state.nodes.is_empty() && !state.data_ready;
    let active_output = state.command_output.active_session();

    {
        let state = state.clone();
        let route_path = route_info.path.clone();
        let first_render = use_mut_ref(|| true);
        use_effect_with(route_path, move |_| {
            if *first_render.borrow() {
                *first_render.borrow_mut() = false;
            } else {
                clear_hash_route();
                state.dispatch(AppEvent::CloseActiveCommandOutput);
            }
        });
    }

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
                    visible={active_output.is_some()}
                    content={active_output.map(|session| session.content.clone()).unwrap_or_default()}
                    command={active_output.map(|session| session.command.clone())}
                    on_close={
                        let state = state.clone();
                        Callback::from(move |_| {
                            clear_hash_route();
                            state.dispatch(AppEvent::CloseActiveCommandOutput);
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
