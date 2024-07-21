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

fn hello() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> String {
        let default = "World";
        let name = state.cache_get_string(HELLO_NAME_CACHE, default);
        if name == default {
            format!("Hello, {default}!")
        } else {
            format!("Hello! The last one here was {name}!\n")
        }
    }
    route("/", get(handler))
}

fn hello_name() -> AppRouter {
    async fn handler(Path(name): Path<String>, State(mut state): State<SharedState>) -> String {
        let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9]+$").unwrap();
        if re.is_match(&name) {
            match state.cache_set_string(HELLO_NAME_CACHE, &name) {
                Ok(_) => {}
                Err(_) => return AppError::Internal("Error caching name".to_string()).to_string(),
            }
            format!("Hello, {}!\n", name)
        } else {
            "Sorry, names must be alphanumeric only.".to_string()
        }
    }
    route("/:name", get(handler))
}
