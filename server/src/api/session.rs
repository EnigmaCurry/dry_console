use crate::{
    api::auth::{Backend, Credentials},
    app_state::SharedState,
    response::{AppJson, JsonResult},
    routing::route,
    AppRouter,
};
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_login::tower_sessions::Session;
use axum_login::AuthSession;
use axum_messages::Messages;
use serde::Serialize;
use tracing::{debug, info, warn};
use utoipa::ToSchema;

const LOGGED_IN_KEY: &str = "logged_in";

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(session())
        .merge(login())
        .merge(logout())
        .merge(read_messages())
        .with_state(SharedState::default())
}

#[derive(Default, Serialize, ToSchema)]
pub struct SessionState {
    /// Is the current user logged in?
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
    async fn handler(session: Session) -> JsonResult<SessionState> {
        let logged_in = is_logged_in(session).await;
        Ok(AppJson(SessionState { logged_in }))
    }
    route("/", get(handler))
}

#[utoipa::path(
    post,
    path = "/api/session/login/",
    responses(
        (status = OK, description = "Logged in", body = SessionState)
    ),
    request_body = Credentials,
)]
fn login() -> AppRouter {
    async fn handler(
        State(state): State<SharedState>,
        session: Session,
        mut auth_session: AuthSession<Backend>,
        Json(creds): Json<Credentials>,
    ) -> impl IntoResponse {
        if is_logged_in(session.clone()).await {
            info!("User already logged in");
            return AppJson(SessionState { logged_in: true }).into_response();
        }
        match state.read() {
            Ok(s) => {
                if !s.is_login_allowed() {
                    warn!("Prevented login attempt - the login service is disabled.");
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "The login service has been disabled. To login again, this service must be restarted.",
                    )
                        .into_response();
                }
            }
            Err(e) => {
                debug!("{:?}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        debug!("{:?}", creds);
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => {
                match state.write() {
                    Ok(mut s) => {
                        // User login is only allowed a single time:
                        s.disable_login();
                        user
                    }
                    Err(e) => {
                        debug!("{:?}", e);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }
            Ok(None) => {
                warn!("Attempted login with invalid username or password");
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
        // Update session with logged in state:
        session.insert(LOGGED_IN_KEY, true).await.unwrap();
        info!("User successfully logged in - now disabling all future logins");
        AppJson(SessionState {
            logged_in: is_logged_in(session).await,
        })
        .into_response()
    }
    route("/login", post(handler))
}

#[utoipa::path(
    post,
    path = "/api/session/logout/",
    responses(
        (status = OK, description = "Logged out", body = SessionState)
    )
)]
fn logout() -> AppRouter {
    async fn handler(
        mut auth_session: AuthSession<Backend>,
        State(state): State<SharedState>,
    ) -> impl IntoResponse {
        let status_code = match auth_session.logout().await {
            Ok(_) => StatusCode::OK,
            Err(e) => {
                debug!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        // Set cookie to expire:
        let headers = [(
            header::SET_COOKIE,
            HeaderValue::from_str("id=; Max-Age=0; Path=/; HttpOnly").unwrap(),
        )];
        (
            status_code,
            headers,
            AppJson(SessionState { logged_in: false }),
        )
            .into_response()
    }
    route("/logout", post(handler))
}

async fn is_logged_in(session: Session) -> bool {
    session
        .get::<bool>(LOGGED_IN_KEY)
        .await
        .expect("could not get session LOGGED_IN_KEY")
        .unwrap_or(false)
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
