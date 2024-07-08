use crate::{
    api::{mod_route, APIModule},
    app_state::SharedState,
};
use axum::{routing::get, routing::MethodRouter, Router};

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .with_state(SharedState::default())
}

fn route(path: &str, method_router: MethodRouter<SharedState>) -> Router<SharedState> {
    mod_route(APIModule::Workstation, path, method_router)
}

fn workstation() -> Router<SharedState> {
    async fn handler() -> &'static str {
        "Workstation"
    }
    route("/", get(handler))
}
