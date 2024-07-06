use crate::api::{mod_route, APIModule};
use axum::{routing::get, routing::MethodRouter, Router};

pub fn router() -> Router {
    Router::new().merge(workstation())
}

fn route(path: &str, method_router: MethodRouter<()>) -> Router {
    mod_route(APIModule::Workstation, path, method_router)
}

fn workstation() -> Router {
    async fn handler() -> &'static str {
        "Workstation"
    }
    route("/", get(handler))
}
