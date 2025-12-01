use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/node/:name/")]
    Node { name: String },
    #[at("/wireguard")]
    WireGuard,
    #[not_found]
    #[at("/404")]
    NotFound,
}
