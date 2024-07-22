use std::collections::HashMap;

use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::Deserialize;
use tracing::debug;
use utoipa::ToSchema;

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
    pub fn add_user(&mut self, username: &str, pw_hash: Vec<u8>) {
        let user = User {
            username: username.to_string(),
            pw_hash,
        };
        self.users.insert(username.to_string(), user);
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
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Credentials {
            username,
            password,
            next,
        }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        debug!("ZZZ USERNAME {username}");
        Ok(self.users.get(&username).cloned())
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

//pub type AuthSession = axum_login::AuthSession<Backend>;
