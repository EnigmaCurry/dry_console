use axum::{
    response::Redirect,
    routing::{get, MethodRouter},
    Router,
};
use enum_iterator::{all, Sequence};

use crate::SharedState;

use super::{mod_route, ApiModule, API_PREFIX};

pub mod counter;
pub mod hello;

const TEST_PREFIX: &str = "test";

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
        let r = format!("/{API_PREFIX}/{TEST_PREFIX}/{}/", self.to_string());
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
    let path_stripped = path.trim_matches('/');
    let path = format!("{path_stripped}/");
    mod_route(
        super::APIModule::Test,
        &format!("/{}/{path}", module.to_string()),
        method_router,
    )
}
