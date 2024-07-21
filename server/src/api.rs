use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{any};
use axum::Router;
use enum_iterator::{all, Sequence};
mod docs;
mod session;
mod test;
mod workstation;

use crate::routing::route;
use crate::{AppMethodRouter, AppRouter};

/// All API modules (and sub-modules) must implement ApiModule trait:
pub trait ApiModule {
    fn main() -> AppRouter;
    fn to_string(&self) -> String;
    fn router(&self) -> AppRouter;
    #[allow(dead_code)]
    fn redirect(&self) -> AppMethodRouter;
}

/// Enumeration of all top-level modules:
#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum APIModule {
    Test,
    Workstation,
    Session,
    Docs,
}

impl ApiModule for APIModule {
    fn main() -> AppRouter {
        // Adds all routes for all modules in APIModule:
        let mut app = Router::new();
        for m in all::<APIModule>() {
            app = app
                .nest(format!("/{}/", m.to_string()).as_str(), m.router())
                // Redirect module URL missing final forward-slash /
                .route(format!("/{}", m.to_string()).as_str(), m.redirect());
        }
        app
    }
    fn router(&self) -> AppRouter {
        match self {
            APIModule::Test => test::router(),
            APIModule::Workstation => workstation::router(),
            APIModule::Session => session::router(),
            APIModule::Docs => docs::router(),
        }
    }
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
    fn redirect(&self) -> AppMethodRouter {
        let r = format!("/{}/", self.to_string());
        any(move || async move { Redirect::permanent(&r) })
    }
}

pub fn router() -> AppRouter {
    // Adds all routes for all modules, and a catch-all for remaining API 404s.
    APIModule::main().route(
        "/*else",
        any(|| async { (StatusCode::NOT_FOUND, "API Not Found") }),
    )
}
