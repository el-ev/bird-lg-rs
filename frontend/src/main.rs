mod app;
mod components;
mod config;
mod hooks;
mod pages;
mod routes;
mod services;
mod store;
mod utils;

fn current_pathname() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

fn main() {
    let config = tracing_wasm::WASMLayerConfigBuilder::new()
        .set_max_level(if cfg!(debug_assertions) {
            tracing::Level::TRACE
        } else {
            tracing::Level::INFO
        })
        .build();
    tracing_wasm::set_as_global_default_with_config(config);

    if autopeer::matches_autopeer_path(&current_pathname()) {
        yew::Renderer::<autopeer::AutoPeerApp>::new().render();
    } else {
        yew::Renderer::<app::App>::new().render();
    }
}
