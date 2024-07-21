use crate::{
    app_state::SharedState,
    auth::{Backend, Credentials},
    response::{AppJson, JsonResult},
};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_login::AuthSession;
use serde::Serialize;
use tracing::debug;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(session))
        .route("/login", post(login))
        .with_state(SharedState::default())
}

#[derive(Default, Serialize)]
struct SessionState {
    logged_in: bool,
}

async fn session() -> JsonResult<SessionState> {
    let s = SessionState::default();
    Ok(AppJson(s))
}

async fn login(
    mut auth_session: AuthSession<Backend>,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            debug!("{:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Redirect::to("/").into_response()
}
