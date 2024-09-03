use super::test_route;

use crate::{
    app_state::SharedState,
    response::{AppError, AppJson, JsonResult},
    AppRouter,
};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::Request,
    routing::{get, MethodRouter},
    Router,
};
use serde::Serialize;

pub fn main() -> AppRouter {
    Router::new()
        .merge(no_error())
        .merge(good_user())
        .merge(bad_user())
}

fn route(path: &str, method_router: MethodRouter<SharedState>) -> AppRouter {
    test_route(super::TestModule::Error, path, method_router)
}

fn no_error() -> AppRouter {
    async fn handler() -> String {
        "No error.".to_string()
    }
    route("/", get(handler))
}

#[derive(Serialize, Clone)]
struct User {
    id: u64,
    name: String,
}

fn good_user() -> AppRouter {
    async fn handler() -> JsonResult<User> {
        Ok(AppJson(User {
            id: 1,
            name: "ryan".to_string(),
        }))
    }
    route("/good_user", get(handler))
}

fn bad_user() -> AppRouter {
    async fn handler(req: Request<Body>) -> JsonResult<User> {
        Err(AppError::Internal(
            anyhow!("bad user"),
            Some(req.uri().to_string()),
        ))
    }
    route("/bad_user", get(handler))
}
