use crate::api::auth::Backend;
use crate::{
    api::route,
    app_state::SharedState,
    response::{AppError, AppJson, JsonResult},
    AppRouter,
};
use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::post, Extension, Router,
};
use axum_login::AuthSession;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use utoipa::ToSchema;

#[derive(Default, Serialize, ToSchema)]
pub struct NewCredentials {
    /// New token for login:
    token: String,
}

pub fn router() -> AppRouter {
    Router::new().merge(shutdown()).merge(enable_login())
}

#[utoipa::path(
    post,
    path = "/api/admin/shutdown/",
    responses(
        (status = OK, description = "Shutdown service", body = str)
    )
)]
fn shutdown() -> AppRouter {
    async fn handler(
        Extension(shutdown_tx): Extension<Arc<Mutex<Option<oneshot::Sender<()>>>>>,
    ) -> impl IntoResponse {
        if let Some(shutdown_tx) = shutdown_tx.lock().await.take() {
            let _ = shutdown_tx.send(());
            (StatusCode::OK, "Server is shutting down")
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server is already shutting down",
            )
        }
    }
    route("/shutdown", post(handler))
}

#[utoipa::path(
    post,
    path = "/api/admin/enable_login/",
    responses(
        (status = OK, description = "Login (re-)enabled", body = NewCredentials)
    ),
)]
fn enable_login() -> AppRouter {
    async fn handler(
        State(state): State<SharedState>,
        auth_session: AuthSession<Backend>,
    ) -> JsonResult<NewCredentials> {
        match state.write() {
            Ok(mut state) => {
                state.enable_login();
                Ok(AppJson(NewCredentials {
                    token: auth_session.backend.get_token(State(state.clone())),
                }))
            }
            Err(e) => Err(AppError::Internal(e.to_string())),
        }
    }
    route("/enable_login", post(handler))
}
