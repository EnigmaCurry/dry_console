use std::convert::Infallible;

use axum::{
    extract::State,
    response::Redirect,
    routing::{any, get, MethodRouter},
    Router,
};
use enum_iterator::{all, Sequence};

use super::{route, APIModule, ApiModule};
use crate::broadcast;
use crate::{app_state::SharedState, AppRouter, API_PREFIX};
pub mod counter;
pub mod error;
pub mod hello;
pub mod ping;

#[derive(Debug, PartialEq, Sequence, Clone)]
enum TestModule {
    Hello,
    Counter,
    Error,
    Ping,
}
impl ApiModule for TestModule {
    fn main(shutdown: broadcast::Sender<()>, state: State<SharedState>) -> AppRouter {
        // Adds all routes for all modules in APIModule:
        let mut app = Router::new();
        for m in all::<TestModule>() {
            app = app.merge(m.router(shutdown.clone(), state.clone()));
        }
        app
    }
    fn router(&self, _shutdown: broadcast::Sender<()>, _state: State<SharedState>) -> AppRouter {
        match self {
            TestModule::Hello => hello::main(),
            TestModule::Counter => counter::main(),
            TestModule::Error => error::main(),
            TestModule::Ping => ping::main(),
        }
    }
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
    fn redirect(&self) -> MethodRouter<SharedState, Infallible> {
        let r = format!(
            "{API_PREFIX}/{}{}/",
            APIModule::Test.to_string(),
            self.to_string()
        );
        any(move || async move { Redirect::permanent(&r) })
    }
}

pub fn router(shutdown: broadcast::Sender<()>, state: State<SharedState>) -> AppRouter {
    TestModule::main(shutdown, state).route("/", get(|| async { "Test" }))
}

fn test_route(
    module: TestModule,
    path: &str,
    method_router: MethodRouter<SharedState, Infallible>,
) -> AppRouter {
    route(
        format!("{}{path}", module.to_string()).as_str(),
        method_router,
    )
}
