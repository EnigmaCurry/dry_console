use crate::SharedState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::{routing::get, routing::MethodRouter, Router};
use text_sanitizer::TextSanitizer;

use super::test_route;

pub fn main() -> Router<SharedState> {
    Router::new().merge(hello()).merge(hello_name())
}

fn route(path: &str, method_router: MethodRouter<SharedState>) -> Router<SharedState> {
    test_route(super::TestModule::Hello, path, method_router)
}

fn hello() -> Router<SharedState> {
    async fn handler(State(state): State<SharedState>) -> String {
        let cache = &state.read().unwrap().cache;
        let default = "World";
        let default_bytes = Bytes::from(default.as_bytes());
        let name = std::str::from_utf8(cache.get("test::hello::name").unwrap_or(&default_bytes))
            .unwrap_or(&default);
        format!("Hello, {name}!\n")
    }
    route("/", get(handler))
}

fn hello_name() -> Router<SharedState> {
    async fn handler(Path(name): Path<String>, State(state): State<SharedState>) -> String {
        let mut sanitizer = TextSanitizer::new_with_options(false, true, false);
        sanitizer.add_request_language(&"en");
        let name: Bytes = Bytes::from(sanitizer.sanitize_u8(name.as_bytes()));
        state
            .write()
            .unwrap()
            .cache
            .insert("test::hello::name".to_string(), name.clone());
        format!(
            "Hello, {}!\n",
            std::str::from_utf8(&name).unwrap_or("World")
        )
    }
    route("/:name", get(handler))
}
