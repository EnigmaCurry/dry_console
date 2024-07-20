use crate::{
    api::{route, APIModule},
    app_state::SharedState,
    auth::{Backend, Credentials},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Json, Router,
};
use axum_login::AuthSession;
use serde_json::json;
use tracing::debug;

pub fn router() -> Router<SharedState> {
    Router::new().with_state(SharedState::default())
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
