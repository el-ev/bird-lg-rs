mod app;
mod components;
mod config;
mod routes;
mod services;
mod store;
mod utils;

pub use common::models;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
