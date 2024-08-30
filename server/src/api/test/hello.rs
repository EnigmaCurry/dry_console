use std::convert::Infallible;

use super::test_route;

use crate::{
    app_state::{ShareableState, SharedState},
    response::AppError,
    AppRouter,
};
use axum::{
    extract::{Path, State},
    routing::{get, MethodRouter},
    Router,
};
use regex::Regex;

const HELLO_NAME_CACHE: &str = "test::hello::name";

pub fn main() -> AppRouter {
    Router::new().merge(hello()).merge(hello_name())
}

fn route(path: &str, method_router: MethodRouter<SharedState, Infallible>) -> AppRouter {
    test_route(super::TestModule::Hello, path, method_router)
}

#[utoipa::path(
    get,
    path = "/api/test/hello/",
    responses(
        (status = OK, description = "Hello", body = str)
    )
)]
fn hello() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> String {
        let default = "World";
        let name = state.cache_get_string(HELLO_NAME_CACHE, default).await;
        if name == default {
            format!("Hello, {default}!")
        } else {
            format!("Hello! The last one here was {name}!\n")
        }
    }
    route("/", get(handler))
}

#[utoipa::path(
    get,
    path = "/api/test/hello/{name}",
    responses(
        (status = OK, description = "Hello name", body = str)
    ),
    params(
        ("name" = str, Path, description="Your name"),
    )
)]
fn hello_name() -> AppRouter {
    async fn handler(Path(name): Path<String>, State(state): State<SharedState>) -> String {
        let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9]+$").unwrap();
        if re.is_match(&name) {
            let mut state = state.write().await;
            state.cache_set_string(HELLO_NAME_CACHE, &name);
            format!("Hello, {}!\n", name)
        } else {
            "Sorry, names must be alphanumeric only.".to_string()
        }
    }
    route("/:name", get(handler))
}
