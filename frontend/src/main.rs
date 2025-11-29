mod app;
mod components;
mod config;
mod hooks;
mod routes;
mod services;
mod store;
mod utils;

fn main() {
    let config = tracing_wasm::WASMLayerConfigBuilder::new()
        .set_max_level(if cfg!(debug_assertions) {
            tracing::Level::TRACE
        } else {
            tracing::Level::INFO
        })
        .build();
    tracing_wasm::set_as_global_default_with_config(config);
    yew::Renderer::<app::App>::new().render();
}
