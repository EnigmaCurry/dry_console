use std::collections::HashMap;

use argon2::{
    password_hash::{
        rand_core::OsRng, Encoding, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use rand::Rng;
use serde::Deserialize;
use tracing::debug;
use utoipa::ToSchema;

use crate::response::AppError;

#[derive(Debug, Clone)]
pub struct User {
    username: String,
    pw_hash: Vec<u8>,
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
    pub fn add_user(&mut self, username: &str, password: &str) {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pw_hash = argon2
            .hash_password(password.as_bytes(), &salt)
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
    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        let argon2 = Argon2::default();
        if let Some(user) = self.users.get(username) {
            let pw_hash = String::from_utf8_lossy(&user.pw_hash);
            match argon2.verify_password(
                password.as_bytes(),
                &PasswordHash::parse(&pw_hash, Encoding::B64).unwrap(),
            ) {
                Ok(()) => return true,
                Err(_) => return false,
            }
        }
        false
    }
}

#[derive(Clone, Deserialize, Debug, ToSchema)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AppError;

    async fn authenticate(
        &self,
        Credentials {
            username,
            password,
            next: _,
        }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match self.users.get(&username) {
            Some(user) => {
                if self.verify_password(&username, &password) {
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
