use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{main_view::MainView, wireguard::WireGuard},
    hooks::use_app_data::use_app_data,
    pages::{MainPage, PeeringPage},
    routes::Route,
    store::{LgState, LgStateHandle, route_info::RouteInfoProvider},
};

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(LgState::default);

    use_app_data(state.clone());

    html! {
        <ContextProvider<LgStateHandle> context={state}>
            <BrowserRouter>
                <RouteInfoProvider>
                    <MainView>
                        <Switch<Route> render={switch} />
                    </MainView>
                </RouteInfoProvider>
            </BrowserRouter>
        </ContextProvider<LgStateHandle>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Root => html! {
            <Redirect<Route> to={Route::Protocols}/>
        },
        Route::Protocols | Route::Node { .. } => html! {
            <MainPage/>
        },
        Route::WireGuard => html! {
            <WireGuard default_open={true}/>
        },
        Route::Peering => html! {
            <PeeringPage/>
        },
        Route::NotFound => {
            html! {}
        }
    }
}
