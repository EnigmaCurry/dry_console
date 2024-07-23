use crate::{
    api::{
        auth::{Backend, Credentials},
        APIModule,
    },
    app_state::SharedState,
    response::{AppError, AppJson, JsonResult},
    routing::route,
    AppRouter,
};
use axum::{
    extract::Form,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use axum_login::AuthSession;
use axum_messages::Messages;
use serde::Serialize;
use tracing::debug;
use utoipa::ToSchema;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(session())
        .merge(login())
        .merge(read_messages())
        .with_state(SharedState::default())
}

#[derive(Default, Serialize, ToSchema)]
pub struct SessionState {
    logged_in: bool,
}

#[derive(Default, Serialize, ToSchema)]
pub struct SessionMessages {
    messages: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/api/session/",
    responses(
        (status = OK, description = "Session state", body = str)
    ),
)]
fn session() -> AppRouter {
    async fn handler() -> JsonResult<SessionState> {
        let s = SessionState::default();
        Ok(AppJson(s))
    }
    route("/", get(handler))
}

#[utoipa::path(
    post,
    path = "/api/session/login/",
    responses(
        (status = OK, description = "Logged in", body = str)
    ),
    request_body = Credentials,
)]
fn login() -> AppRouter {
    async fn handler(
        mut auth_session: AuthSession<Backend>,
        Json(creds): Json<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return StatusCode::UNAUTHORIZED.into_response();
            }
            Err(e) => {
                debug!("{:?}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        AppJson(SessionState { logged_in: true }).into_response()
    }
    route("/login", post(handler))
}

#[utoipa::path(
    get,
    path = "/api/session/messages/",
    responses(
        (status = OK, description = "Read messages", body = str)
    ),
)]
fn read_messages() -> AppRouter {
    async fn handler(messages: Messages) -> impl IntoResponse {
        let messages = messages
            .into_iter()
            .map(|message| format!("{}: {}", message.level, message))
            .collect::<Vec<_>>();

        AppJson(SessionMessages { messages })
    }
    route("/messages", get(handler))
}
