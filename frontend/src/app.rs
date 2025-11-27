use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{
    content_modal::ContentModal, node_list::NodeList, route_lookup::RouteLookup,
    status_banner::StatusBanner, traceroute::Traceroute,
};
use crate::config::load_config;
use crate::routes::Route;
use crate::services::api::Api;
use crate::services::websocket::WebSocketService;
use crate::store::modal::ModalAction;
use crate::store::{Action, AppState};
use crate::utils::{log_error, sleep_ms};

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
            Api::get_protocol_details(&state, node, proto);
        })
    };

    let on_route_lookup = {
        let state = state.clone();
        Callback::from(move |(node, target, all): (String, String, bool)| {
            Api::route_lookup(&state, node, target, all);
        })
    };

    let waiting_for_data = state.nodes.is_empty() && (!state.data_ready || !state.config_ready);
    let fetch_error = props
        .error
        .clone()
        .or(node_not_found_error)
        .or(state.fetch_error.clone());

    html! {
        <main class="hero">
            <div class="container">
                <h2 class="title title-flex">
                    <Link<Route> to={Route::Home} classes="title-link">{"Looking Glass"}</Link<Route>>
                    {
                        html! {
                            <span class="title-footnote">
                                if let Some(ref info) = state.network_info {
                                    { " of " } { &info.name } { " " } { &info.asn } {" on DN42 "}
                                }
                                if let Some(name) = &props.node_name {
                                    if !nodes.is_empty() {
                                        { " / " } { name }
                                    }
                                }
                            </span>
                        }
                    }
                </h2>

                <StatusBanner
                    fetch_error={fetch_error}
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

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match load_config().await {
                    Ok(config) => {
                        state.dispatch(Action::SetConfig {
                            username: config.username.clone(),
                            backend_url: config.backend_url.clone(),
                        });
                    }
                    Err(err) => {
                        let message = format!("Configuration load failed: {}", err);
                        state.dispatch(Action::SetError(message.clone()));
                        log_error(&message);
                    }
                }
            });

            || ()
        });
    }

    {
        let state = state.clone();
        use_effect_with(state.config_ready, move |ready| {
            if *ready {
                let state_info = state.clone();
                spawn_local(async move {
                    if let Err(e) = Api::get_network_info(&state_info).await {
                        log_error(&e);
                    }
                });

                let backend_url = state.backend_url.clone();
                let state_ws = state.clone();
                spawn_local(async move {
                    WebSocketService::connect(backend_url, state_ws);
                });
            }

            || ()
        });
    }

    {
        let state = state.clone();
        use_effect_with(
            (state.config_ready, state.ws_sender.is_some()),
            move |(config_ready, ws_connected)| {
                let active = Rc::new(RefCell::new(true));

                if *config_ready && !*ws_connected {
                    let state = state.clone();
                    let active = active.clone();
                    spawn_local(async move {
                        loop {
                            sleep_ms(5000).await;
                            if !*active.borrow() {
                                break;
                            }
                            if let Err(e) = Api::get_protocols(&state).await {
                                log_error(&e);
                            }
                        }
                    });
                }

                move || {
                    *active.borrow_mut() = false;
                }
            },
        );
    }

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
