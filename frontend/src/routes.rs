use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Root,
    #[at("/protocols")]
    Protocols,
    #[at("/node/:name/")]
    Node { name: String },
    #[at("/peering")]
    Peering,
    #[at("/wireguard")]
    WireGuard,
    #[not_found]
    #[at("/404")]
    NotFound,
}
