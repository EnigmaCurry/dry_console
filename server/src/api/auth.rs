use crate::{
    app_state::{AppState, SharedState},
    response::AppError,
};
use async_trait::async_trait;
use axum::extract::State;
use axum_login::{AuthUser, AuthnBackend, UserId};
use std::fmt;
use tracing::debug;
//use jiff::{Timestamp, Zoned};
use serde::Deserialize;
//use tracing::debug;
use utoipa::ToSchema;

use super::token::generate_token;

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
    pub fn reset_token(&mut self, State(state): State<SharedState>) -> String {
        debug!("reset_token() ...");
        match state.write() {
            Ok(mut s) => {
                s.cache_set_string(TOKEN_CACHE_NAME, &generate_token());
                match s.cache_get_string(TOKEN_CACHE_NAME, "").as_str() {
                    "" => panic!("Could not retrieve the token cache entry just set?!"),
                    q => q.to_string(),
                }
            }
            Err(e) => panic!("Failed to reset token: {:?}", e),
        }
    }
    pub fn verify_token(&self, token: &str, State(state): State<SharedState>) -> bool {
        token
            == state
                .read()
                .expect("Could not read state")
                .cache_get_string(TOKEN_CACHE_NAME, &generate_token())
    }
    pub fn get_token(&self, State(state): State<AppState>) -> String {
        state.cache_get_string(TOKEN_CACHE_NAME, &generate_token())
    }
    pub fn get_state(&self) -> State<SharedState> {
        self.state.clone()
    }
}

#[derive(Clone, Deserialize, ToSchema)]
pub struct Credentials {
    /// One time token for login
    #[schema(example = "")]
    pub token: String,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("token", &"REDACTED")
            .finish()
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
        if self.verify_token(&token, self.state.clone()) {
            Ok(Some(self.user.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(Some(self.user.clone()))
    }
}
