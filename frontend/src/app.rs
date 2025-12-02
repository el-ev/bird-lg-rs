use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::main_view::MainView;
use crate::components::wireguard::WireGuard;
use crate::hooks::use_app_data::use_app_data;
use crate::pages::{MainPage, PeeringPage};
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
