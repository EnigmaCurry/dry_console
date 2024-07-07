use super::test_route;
use crate::SharedState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::{routing::get, routing::MethodRouter, Router};
use regex::Regex;

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
        if name == default {
            format!("Hello, {default}!")
        } else {
            format!("Hello! The last one here was {name}!\n")
        }
    }
    route("/", get(handler))
}

fn hello_name() -> Router<SharedState> {
    async fn handler(Path(name): Path<String>, State(state): State<SharedState>) -> String {
        let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9]+$").unwrap();
        if re.is_match(&name) {
            state
                .write()
                .unwrap()
                .cache
                .insert("test::hello::name".to_string(), Bytes::from(name.clone()));
            format!("Hello, {}!\n", name)
        } else {
            format!("Sorry, names must be alphanumeric only.")
        }
    }
    route("/:name", get(handler))
}
