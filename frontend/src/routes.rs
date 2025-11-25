use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/:name/")]
    Node { name: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}
