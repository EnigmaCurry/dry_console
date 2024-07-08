use crate::{api::route, app_state::SharedState};
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
    route("/", get(handler))
}

fn workstation_foo() -> Router<SharedState> {
    async fn handler() -> &'static str {
        "Workstation foo"
    }
    route("/foo", get(handler))
}
