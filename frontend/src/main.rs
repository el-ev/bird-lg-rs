mod app;
mod components;
mod config;
mod models;
mod services;
mod store;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
