use super::test_route;

use crate::{AppMethodRouter, AppRouter};
use axum::Router;

const HELLO_NAME_CACHE: &str = "test::hello::name";

pub fn main() -> AppRouter {
    Router::new() //.merge(hello()).merge(hello_name())
}

fn route(path: &str, method_router: AppMethodRouter) -> AppRouter {
    test_route(super::TestModule::Hello, path, method_router)
}

fn hello() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> String {
        let default = "World";
        let name = match state.cache_get_string(HELLO_NAME_CACHE, default) {
            Ok(s) => s,
            Err(_) => default.to_string(),
        };
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
                Err(e) => {}
            }
            format!("Hello, {}!\n", name)
        } else {
            format!("Sorry, names must be alphanumeric only.")
        }
    }
    route("/:name", get(handler))
}
