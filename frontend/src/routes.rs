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
    #[at("/dn42")]
    Dn42,
    #[at("/autopeer")]
    AutoPeer,
    #[not_found]
    #[at("/404")]
    NotFound,
}
