use crate::response::AppError;
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use sha2::{Digest, Sha256};
use std::fmt;
//use jiff::{Timestamp, Zoned};
use serde::Deserialize;
//use tracing::debug;
use utoipa::ToSchema;

use super::token::{self, generate_token};

const ADMIN_USER: &str = "admin";
const TOKEN_EXPIRATION_MINUTES: i64 = 60;
const PLACEHOLDER_SALT: &str = "default";

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
    secret: Vec<u8>,
}
impl Backend {
    pub fn new(secret: &[u8]) -> Self {
        return Self {
            user: User::default(),
            secret: secret[..32].to_vec(),
        };
    }
    pub fn reset_token(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        self.user = User {};
        generate_token(&self.secret, TOKEN_EXPIRATION_MINUTES)
    }
    pub fn verify_token(&self, token: &str) -> bool {
        token::validate_token(&token, &self.secret).unwrap_or(false)
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

pub fn derive_key(input: &[u8]) -> [u8; 32] {
    fn derive_key_with_salt(input: &[u8], salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.update(salt);
        let result = hasher.finalize();
        let mut secret = [0u8; 32];
        secret.copy_from_slice(&result);
        secret
    }
    derive_key_with_salt(input, PLACEHOLDER_SALT.as_bytes())
}
