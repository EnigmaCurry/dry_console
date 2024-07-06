use axum::extract::Path;
use axum::{routing::get, routing::MethodRouter, Router};

use super::test_route;

pub fn main() -> Router {
    Router::new().merge(hello()).merge(hello_name())
}

fn route(path: &str, method_router: MethodRouter<()>) -> Router {
    test_route(super::TestModule::Hello, path, method_router)
}

fn hello() -> Router {
    async fn handler() -> &'static str {
        "Hello, World!\n"
    }
    route("/", get(handler))
}

fn hello_name() -> Router {
    async fn handler(Path(name): Path<String>) -> String {
        format!("Hello, {}!\n", name)
    }
    route("/:name", get(handler))
}
