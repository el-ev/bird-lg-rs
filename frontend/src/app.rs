use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{
    content_modal::ContentModal, header::Header, node_list::NodeList, route_lookup::RouteLookup,
    status_banner::StatusBanner, traceroute::Traceroute,
};
use crate::hooks::use_app_data::use_app_data;
use crate::routes::Route;
use crate::services::api::{get_protocol_details, perform_route_lookup};
use crate::store::modal::ModalAction;
use crate::store::{Action, AppState};

#[derive(Properties, PartialEq)]
pub struct MainViewProps {
    pub node_name: Option<String>,
    #[prop_or_default]
    pub error: Option<String>,
}

#[function_component(MainView)]
pub fn main_view(props: &MainViewProps) -> Html {
    let state = use_context::<UseReducerHandle<AppState>>().expect("no app state found");

    let (nodes, node_not_found_error) = if let Some(name) = &props.node_name {
        let filtered: Vec<_> = state
            .nodes
            .iter()
            .filter(|n| n.name == *name)
            .cloned()
            .collect();

        if filtered.is_empty() && state.data_ready {
            (filtered, Some(format!("Node '{}' not found", name)))
        } else {
            (filtered, None)
        }
    } else {
        (state.nodes.clone(), None)
    };

    let on_protocol_click = {
        let state = state.clone();
        Callback::from(move |(node, proto): (String, String)| {
            get_protocol_details(&state, node, proto);
        })
    };

    let on_route_lookup = {
        let state = state.clone();
        Callback::from(move |(node, target, all): (String, String, bool)| {
            perform_route_lookup(&state, node, target, all);
        })
    };

    let waiting_for_data = state.nodes.is_empty() && (!state.data_ready || !state.config_ready);
    let error = props
        .error
        .clone()
        .or(node_not_found_error)
        .or(state.fetch_error.clone());

    html! {
        <main class="hero">
            <div class="container">
                <Header
                    node_name={props.node_name.clone()}
                    network_info={state.network_info.clone()}
                    nodes_count={nodes.len()}
                />

                <StatusBanner
                    error={error}
                    waiting_for_data={waiting_for_data}
                />

                if state.config_ready {
                    <>
                        <NodeList
                            state={state.clone()}
                            nodes={nodes.clone()}
                            on_protocol_click={on_protocol_click}
                        />

                        <Traceroute
                            state={state.clone()}
                            nodes={nodes.clone()}
                        />

                        <RouteLookup
                            state={state.clone()}
                            nodes={nodes.clone()}
                            on_lookup={on_route_lookup}
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
                    </>
                }
            </div>
        </main>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(AppState::default);

    use_app_data(state.clone());

    html! {
        <ContextProvider<UseReducerHandle<AppState>> context={state}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<UseReducerHandle<AppState>>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <MainView node_name={None::<String>} /> },
        Route::Node { name } => html! { <MainView node_name={Some(name)} /> },
        Route::NotFound => html! {
            <MainView
                node_name={Some("".to_string())}
                error={Some("Page not found".to_string())}
            />
        },
    }
}
