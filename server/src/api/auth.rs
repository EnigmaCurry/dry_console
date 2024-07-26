use crate::response::AppError;
use argon2::{
    password_hash::{
        rand_core::OsRng, Encoding, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use std::collections::HashMap;
use std::fmt;
//use jiff::{Timestamp, Zoned};
use serde::Deserialize;
use tracing::debug;
use utoipa::ToSchema;

#[derive(Clone)]
pub struct User {
    pub username: String,
    pw_hash: Vec<u8>,
}
impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("username", &self.username)
            .field("pw_hash", &"REDACTED")
            .finish()
    }
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

#[derive(Clone, Default)]
pub struct Backend {
    users: HashMap<String, User>,
}
impl Backend {
    pub fn add_user(&mut self, username: &str, token: &str) {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pw_hash = argon2
            .hash_password(token.as_bytes(), &salt)
            .unwrap()
            .to_string()
            .as_bytes()
            .to_vec();
        let user = User {
            username: username.to_string(),
            pw_hash,
        };
        self.users.insert(username.to_string(), user);
    }
    pub fn verify_token(&self, username: &str, token: &str) -> bool {
        let argon2 = Argon2::default();
        if let Some(user) = self.users.get(username) {
            debug!("{:?}", user);
            let pw_hash = String::from_utf8_lossy(&user.pw_hash);
            match argon2.verify_password(
                token.as_bytes(),
                &PasswordHash::parse(&pw_hash, Encoding::B64).unwrap(),
            ) {
                Ok(()) => return true,
                Err(_) => return false,
            }
        }
        false
    }
}

#[derive(Clone, Deserialize, ToSchema)]
pub struct Credentials {
    #[serde(default = "default_username")]
    /// Username for login
    #[schema(example = "admin")]
    pub username: String,
    /// One time token for login
    #[schema(example = "")]
    pub token: String,
}
fn default_username() -> String {
    "admin".to_string()
}
impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
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
        Credentials { username, token }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match self.users.get(&username) {
            Some(user) => {
                if self.verify_token(&username, &token) {
                    Ok(Some(user).cloned())
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

//pub type AuthSession = axum_login::AuthSession<Backend>;
