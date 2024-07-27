use std::convert::Infallible;

use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{any, get, MethodRouter};
use axum::Router;
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use axum_login::{login_required, AuthManagerLayerBuilder};
use axum_messages::MessagesManagerLayer;
use enum_iterator::{all, Sequence};
use tracing::info;
mod admin;
mod auth;
mod docs;
mod session;
mod test;
mod token;
mod workstation;
use crate::api::auth::Backend;
use crate::app_state::SharedState;
use crate::routing::route;
use crate::AppRouter;

/// All API modules (and sub-modules) must implement ApiModule trait:
pub trait ApiModule {
    fn main() -> AppRouter;
    fn to_string(&self) -> String;
    fn router(&self) -> AppRouter;
    #[allow(dead_code)]
    fn redirect(&self) -> MethodRouter<SharedState, Infallible>;
}

/// Enumeration of all top-level modules:
#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum APIModule {
    Admin,
    Test,
    Workstation,
}
impl ApiModule for APIModule {
    fn main() -> AppRouter {
        // Adds all routes for all modules in APIModule:
        let mut app = Router::new();
        for m in all::<APIModule>() {
            app = app.nest(format!("/{}/", m.to_string()).as_str(), m.router())
        }
        app
    }
    fn router(&self) -> AppRouter {
        match self {
            APIModule::Admin => admin::router(),
            APIModule::Test => test::router(),
            APIModule::Workstation => workstation::router(),
        }
    }
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
    fn redirect(&self) -> MethodRouter<SharedState, Infallible> {
        let r = format!("/{}/", self.to_string());
        any(move || async move { Redirect::permanent(&r) })
    }
}

///Adds all routes for all API modules
pub fn router() -> AppRouter {
    let key = cookie::Key::generate();
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_signed(key.clone());
    let mut auth_backend = auth::Backend::new();

    let token = auth_backend.reset_token();
    info!("Login credentials::\nToken: {}", token);
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer.clone()).build();
    APIModule::main()
        .route_layer(login_required!(Backend))
        .nest("/session/", session::router())
        .layer(MessagesManagerLayer)
        .layer(auth_layer)
        // everything above auth_layer is private and requires authentication
        // everything after auth_layer is public and requires no authentication
        .layer(session_layer)
        .nest("/docs/", docs::router())
        .route(
            "/unprotected",
            get(|| async { "Hi there, this page is unprotected!" }),
        )
        .route(
            "/*else",
            any(|| async { (StatusCode::NOT_FOUND, "API Not Found") }),
        )
}
