use yew::UseReducerHandle;

pub mod lg_state;
pub mod modal;
pub mod route_info;
pub mod traceroute;

pub use lg_state::{Action, LgState};
pub use traceroute::NodeTracerouteResult;

pub type AppStateHandle = UseReducerHandle<LgState>;
