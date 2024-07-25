use std::convert::Infallible;

use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{any, get, MethodRouter};
use axum::Router;
//use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use axum_login::{login_required, AuthManagerLayerBuilder};
use axum_messages::MessagesManagerLayer;
use enum_iterator::{all, Sequence};
use tracing::info;
mod auth;
mod docs;
mod random;
mod session;
mod test;
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
        //.with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);
    let mut auth_backend = auth::Backend::default();

    let admin_password = random::generate_secure_passphrase(16);
    auth_backend.add_user("admin", admin_password.as_str());
    info!(
        "Login credentials::\nUsername: admin\nPassword: {}",
        admin_password
    );
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer.clone()).build();
    APIModule::main()
        .route(
            "/protected",
            get(|| async { "Gotta be logged in to see me!" }),
        )
        .route(
            "/also-protected",
            get(|| async { "Gotta be logged in to see me!" }),
        )
        .route_layer(login_required!(Backend))
        .nest("/session/", session::router())
        .layer(MessagesManagerLayer)
        .layer(auth_layer)
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
