use axum::routing::get;
use axum::Router;
use axum::{response::Redirect, routing::MethodRouter};
use enum_iterator::{all, Sequence};

use crate::app_state::SharedState;

mod test;
mod workstation;

/// All API modules (and sub-modules) must implement ApiModule trait:
pub trait ApiModule {
    fn main() -> Router<SharedState>;
    fn to_string(&self) -> String;
    fn router(&self) -> Router<SharedState>;
    #[allow(dead_code)]
    fn redirect(&self) -> MethodRouter;
}

/// Enumeration of all top-level modules:
#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum APIModule {
    Test,
    Workstation,
}
impl ApiModule for APIModule {
    fn main() -> Router<SharedState> {
        // Adds all routes for all modules in APIModule:
        let mut app = Router::new();
        for m in all::<APIModule>() {
            app = app.nest(format!("/{}/", m.to_string()).as_str(), m.router());
        }
        // Return merge router with a final fallback for 404:
        // app.route("/*else")
        app
    }
    fn router(&self) -> Router<SharedState> {
        match self {
            APIModule::Test => test::router(),
            APIModule::Workstation => workstation::router(),
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
    // Adds all routes for all modules in APIModule:
    let r = APIModule::main();
    r
}

fn route(path: &str, method_router: MethodRouter<SharedState>) -> Router<SharedState> {
    let p: String;
    match path.trim_matches('/') {
        "" => {
            p = "/".to_string();
        }
        p2 => p = format!("/{}/", p2.to_string()),
    }
    Router::new().route(&p, method_router)
}
