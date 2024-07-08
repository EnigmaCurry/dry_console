use crate::{
    api::{route, APIModule},
    app_state::SharedState,
};
use axum::{routing::get, Router};

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .merge(workstation_foo())
        .with_state(SharedState::default())
}

fn workstation() -> Router<SharedState> {
    async fn handler() -> &'static str {
        "Workstation"
    }
    route(APIModule::Workstation, "/", get(handler))
}

fn workstation_foo() -> Router<SharedState> {
    async fn handler() -> &'static str {
        "Workstation foo"
    }
    route(APIModule::Workstation, "/foo", get(handler))
}
