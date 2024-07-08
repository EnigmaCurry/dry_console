use axum::http::StatusCode;
use axum::routing::any;
use axum::Router;
use axum::{response::Redirect, routing::MethodRouter};
use enum_iterator::{all, Sequence};

use crate::app_state::SharedState;

mod test;
mod workstation;

use crate::routing::route;
use crate::API_PREFIX;

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
            app = app
                .nest(format!("/{}/", m.to_string()).as_str(), m.router())
                // Redirect module URL missing final forward-slash /
                .route(
                    format!("/{}", m.to_string()).as_str(),
                    any(Redirect::permanent(
                        format!("{API_PREFIX}/{}/", m.to_string()).as_str(),
                    )),
                );
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
        any(move || async move { Redirect::permanent(&r) })
    }
}

pub fn router() -> Router<SharedState> {
    // Adds all routes for all modules, and a catch-all for remaining API 404s.
    APIModule::main().route(
        "/*else",
        any(|| async { (StatusCode::NOT_FOUND, "API Not Found") }),
    )
}
