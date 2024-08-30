use super::token::generate_token;
use crate::{
    app_state::{AppState, SharedState},
    response::AppError,
};
use async_trait::async_trait;
use axum::extract::State;
use axum_login::{AuthUser, AuthnBackend, UserId};
pub use dry_console_dto::session::Credentials;
use tracing::debug;

pub const TOKEN_CACHE_NAME: &str = "token";
const ADMIN_USER: &str = "admin";

#[derive(Clone, Debug, Default)]
pub struct User {
    //There is only one user (ADMIN_USER) so this needs no data.
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        // single static id: admin
        ADMIN_USER.to_string()
    }

    fn session_auth_hash(&self) -> &[u8] {
        // Single session for the admin user:
        ADMIN_USER.as_bytes()
    }
}

#[derive(Clone)]
pub struct Backend {
    state: State<SharedState>,
    user: User,
}
impl Backend {
    pub fn new(state: &SharedState) -> Self {
        Self {
            user: User::default(),
            state: State(state.clone()),
        }
    }
    pub async fn reset_token(&mut self, State(state): State<SharedState>) -> String {
        {
            debug!("reset_token() ...");
            let mut state = state.write().await;
            debug!("nooope");
            state.cache_set_string(TOKEN_CACHE_NAME, &generate_token());
        }
        {
            let state = state.read().await;
            match state.cache_get_string(TOKEN_CACHE_NAME, "").as_str() {
                "" => panic!("Could not retrieve the token cache entry just set?!"),
                q => q.to_string(),
            }
        }
    }
    pub async fn verify_token(&self, token: &str, State(state): State<SharedState>) -> bool {
        token
            == state
                .read()
                .await
                .cache_get_string(TOKEN_CACHE_NAME, &generate_token())
    }
    pub fn get_token(&self, state: AppState) -> String {
        state.cache_get_string(TOKEN_CACHE_NAME, &generate_token())
    }
    pub fn get_state(&self) -> State<SharedState> {
        self.state.clone()
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AppError;

    async fn authenticate(
        &self,
        Credentials { token }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if self.verify_token(&token, self.state.clone()).await {
            Ok(Some(self.user.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(Some(self.user.clone()))
    }
}
