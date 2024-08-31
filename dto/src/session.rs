use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, ToSchema)]
pub struct SessionState {
    /// Is the current user logged in?
    pub logged_in: bool,
    /// Are new logins allowed?
    pub new_login_allowed: bool,
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct SessionMessages {
    pub messages: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
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
