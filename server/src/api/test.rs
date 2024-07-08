use axum::{
    response::Redirect,
    routing::{get, MethodRouter},
    Router,
};
use enum_iterator::{all, Sequence};

use crate::app_state::SharedState;

use super::{route, ApiModule};

pub mod counter;
pub mod hello;

#[derive(Debug, PartialEq, Sequence, Clone)]
enum TestModule {
    Hello,
    Counter,
}
impl ApiModule for TestModule {
    fn main() -> Router<SharedState> {
        // Adds all routes for all modules in APIModule:
        let mut app = Router::new();
        for m in all::<TestModule>() {
            app = app.merge(m.router());
        }
        app
    }
    fn router(&self) -> Router<SharedState> {
        match self {
            TestModule::Hello => hello::main(),
            TestModule::Counter => counter::main(),
        }
    }
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
    fn redirect(&self) -> MethodRouter {
        let r = format!("/{}/", self.to_string());
        get(move || async move { Redirect::permanent(&r) })
    }
}

pub fn router() -> Router<SharedState> {
    TestModule::main()
}

fn test_route(
    module: TestModule,
    path: &str,
    method_router: MethodRouter<SharedState>,
) -> Router<SharedState> {
    route(
        format!("{}{path}", module.to_string()).as_str(),
        method_router,
    )
}
