use crate::response::AppError;
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use std::fmt;
//use jiff::{Timestamp, Zoned};
use serde::Deserialize;
//use tracing::debug;
use utoipa::ToSchema;

use super::token::generate_token;

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
    user: User,
    token: String,
}
impl Backend {
    pub fn new() -> Self {
        Self {
            user: User::default(),
            token: generate_token(),
        }
    }
    pub fn reset_token(&mut self) -> String {
        self.token = generate_token();
        self.token.clone()
    }
    pub fn verify_token(&self, token: &str) -> bool {
        token == self.token
    }
    pub fn get_token(&self) -> String {
        self.token.clone()
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
        if self.verify_token(&token) {
            Ok(Some(self.user.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(Some(self.user.clone()))
    }
}
