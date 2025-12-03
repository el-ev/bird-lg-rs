use std::{cell::RefCell, rc::Rc};

use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    config::{Config, load_config},
    services::{
        api::{get_network_info, get_protocols},
        websocket::WebSocketService,
    },
    store::{Action, LgStateHandle},
    utils::sleep_ms,
};

#[hook]
pub fn use_app_data(state: LgStateHandle) {
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            if let Ok(config) = LocalStorage::get::<Config>("app_config") {
                state.dispatch(Action::SetConfig {
                    username: config.username.clone(),
                    backend_url: config.backend_url.clone(),
                });
            }

            spawn_local(async move {
                match load_config().await {
                    Ok(config) => {
                        if let Err(e) = LocalStorage::set("app_config", &config) {
                            tracing::error!("Failed to cache config: {}", e);
                        }

                        state.dispatch(Action::SetConfig {
                            username: config.username.clone(),
                            backend_url: config.backend_url.clone(),
                        });
                    }
                    Err(err) => {
                        let message = format!("Configuration load failed: {}", err);
                        state.dispatch(Action::SetError(message.clone()));
                        tracing::error!("{}", message);
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
                    if let Err(e) = get_network_info(&state_info).await {
                        tracing::error!("{}", e);
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
                            if let Err(e) = get_protocols(&state).await {
                                tracing::error!("{}", e);
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
}
