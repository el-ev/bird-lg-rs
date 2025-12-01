use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::main_view::MainView;
use crate::components::wireguard::WireGuard;
use crate::hooks::use_app_data::use_app_data;
use crate::pages::ProtocolPage;
use crate::routes::Route;
use crate::store::route_info::RouteInfoProvider;
use crate::store::{AppStateHandle, LgState};

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(LgState::default);

    use_app_data(state.clone());

    html! {
        <ContextProvider<AppStateHandle> context={state}>
            <BrowserRouter>
                <RouteInfoProvider>
                    <MainView>
                        <Switch<Route> render={switch} />
                    </MainView>
                </RouteInfoProvider>
            </BrowserRouter>
        </ContextProvider<AppStateHandle>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home | Route::Node { .. } => html! {
            <ProtocolPage/>
        },
        Route::WireGuard => html! {
            <WireGuard/>
        },
        Route::NotFound => {
            html! {}
        }
    }
}
