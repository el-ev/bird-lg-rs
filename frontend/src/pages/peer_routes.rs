use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{routes::Route, services::api::perform_peer_routes, store::LgStateHandle};

#[function_component(PeerRoutesPage)]
pub fn peer_routes_page() -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let route = use_route::<Route>().unwrap_or(Route::NotFound);

    let (node, peer) = match route {
        Route::PeerRoutes { node, peer } => (node, peer),
        _ => return html! {},
    };

    {
        let state = state.clone();
        let node = node.clone();
        let peer = peer.clone();
        use_effect_with(peer.clone(), move |_| {
            spawn_local(async move {
                if let Err(error) = perform_peer_routes(&state, node, peer).await {
                    tracing::error!("Peer routes lookup failed: {}", error);
                }
            });
        });
    }

    html! {
        <div>
            <h3>
                {"Routes from "}
                <Link<Route> to={Route::Node { name: node.clone() }}>{ &node }</Link<Route>>
                {format!(" / {}", peer)}
            </h3>
            <p class="status-message">
                {"Showing routes received via protocol "}
                <code>{ &peer }</code>
                {". Output is displayed in the command overlay."}
            </p>
        </div>
    }
}
