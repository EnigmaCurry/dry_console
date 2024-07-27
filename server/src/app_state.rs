use crate::api::auth::TOKEN_CACHE_NAME;
use crate::api::token::generate_token;
use crate::response::AppError;
use axum::body::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

////////////////////////////////////////////////////////////////////////////////
// Global app state
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug)]
pub struct AppState {
    cache: HashMap<String, Bytes>,
    login_allowed: bool,
}
impl Default for AppState {
    fn default() -> Self {
        let token = generate_token();
        let s = AppState {
            cache: HashMap::from([(TOKEN_CACHE_NAME.to_string(), Bytes::from(token.clone()))]),
            login_allowed: true,
        };
        info!("\n\nLogin credential generated:\nToken: {}\n", token);
        s
    }
}
impl AppState {
    pub fn cache_set(&mut self, key: &str, value: &Bytes) {
        self.cache.insert(key.to_string(), value.clone());
    }
    pub fn cache_set_string(&mut self, key: &str, value: &str) {
        self.cache_set(key, &Bytes::from(value.to_string()));
    }
    pub fn cache_get(&self, key: &str, default: &Bytes) -> Bytes {
        self.cache.get(key).unwrap_or(&default.clone()).clone()
    }
    pub fn cache_get_string(&self, key: &str, default: &str) -> String {
        std::str::from_utf8(&self.cache_get(key, &Bytes::from(default.to_string())))
            .unwrap_or(default)
            .to_string()
    }
    pub fn is_login_allowed(&self) -> bool {
        self.login_allowed
    }
    pub fn disable_login(&mut self) {
        self.login_allowed = false;
    }
    pub fn enable_login(&mut self) {
        self.login_allowed = true;
    }
}
pub type SharedState = Arc<RwLock<AppState>>;

pub trait ShareableState {
    #[allow(dead_code)]
    fn cache_set(&mut self, key: &str, value: &Bytes) -> Result<(), AppError>;
    fn cache_set_string(&mut self, key: &str, value: &str) -> Result<(), AppError>;
    #[allow(dead_code)]
    fn cache_get(&self, key: &str, default: &Bytes) -> Bytes;
    fn cache_get_string(&self, key: &str, default: &str) -> String;
}
impl ShareableState for SharedState {
    fn cache_set(&mut self, key: &str, value: &Bytes) -> Result<(), AppError> {
        //Locks the entire hashmap to do a single atomic write:
        match self.write() {
            Ok(mut state) => {
                state.cache_set(key, value);
                Ok(())
            }
            Err(e) => Err(AppError::from(e)),
        }
    }

    fn cache_set_string(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        //Locks the entire hashmap to do a single atomic write:
        match self.write() {
            Ok(mut state) => {
                state.cache_set_string(key, value);
                Ok(())
            }
            Err(e) => Err(AppError::from(e)),
        }
    }

    fn cache_get(&self, key: &str, default: &Bytes) -> Bytes {
        //Block only if there is an atomic write in progress,
        //otherwise, multiple readers can read at the same time:
        match self.read() {
            Ok(state) => state.cache_get(key, default),
            Err(e) => {
                debug!("{:?}", e);
                default.clone()
            }
        }
    }

    fn cache_get_string(&self, key: &str, default: &str) -> String {
        //Block only if there is an atomic write in progress,
        //otherwise, multiple readers can read at the same time:
        match self.read() {
            Ok(state) => state.cache_get_string(key, default),
            Err(e) => {
                debug!("{:?}", e);
                default.to_string()
            }
        }
    }
}
