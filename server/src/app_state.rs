use axum::body::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::response::{AppError, AppError::SharedStateError};

////////////////////////////////////////////////////////////////////////////////
// Global app state
////////////////////////////////////////////////////////////////////////////////
#[derive(Default)]
pub struct AppState {
    cache: HashMap<String, Bytes>,
}
impl AppState {
    pub fn cache_set(&mut self, key: &str, value: &Bytes) {
        self.cache.insert(key.to_string(), value.clone());
    }
    pub fn cache_set_string(&mut self, key: &str, value: &str) {
        self.cache_set(key, &Bytes::from(value.to_string()));
    }
    pub fn cache_get(&self, key: &str, default: &Bytes) -> Bytes {
        self.cache
            .get(key)
            .unwrap_or(&Bytes::from(default.clone()))
            .clone()
    }
    pub fn cache_get_string(&self, key: &str, default: &str) -> String {
        std::str::from_utf8(&self.cache_get(key, &Bytes::from(default.to_string())))
            .unwrap_or(&default)
            .to_string()
    }
}
pub type SharedState = Arc<RwLock<AppState>>;
pub trait ShareableState {
    fn cache_set(&mut self, key: &str, value: &Bytes) -> Result<(), AppError>;
    fn cache_set_string(&mut self, key: &str, value: &str) -> Result<(), AppError>;
    fn cache_get(&self, key: &str, default: &Bytes) -> Result<Bytes, AppError>;
    fn cache_get_string(&self, key: &str, default: &str) -> Result<String, AppError>;
}
impl ShareableState for SharedState {
    fn cache_set(&mut self, key: &str, value: &Bytes) -> Result<(), AppError> {
        match self.write() {
            Ok(mut state) => {
                state.cache_set(key, value);
                Ok(())
            }
            Err(e) => Err(SharedStateError(e.to_string())),
        }
    }
    fn cache_set_string(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        match self.write() {
            Ok(mut state) => {
                state.cache_set_string(key, value);
                Ok(())
            }
            Err(e) => Err(SharedStateError(e.to_string())),
        }
    }
    fn cache_get(&self, key: &str, default: &Bytes) -> Result<Bytes, AppError> {
        match self.read() {
            Ok(state) => Ok(state.cache_get(key, default)),
            Err(e) => Err(SharedStateError(e.to_string())),
        }
    }
    fn cache_get_string(&self, key: &str, default: &str) -> Result<String, AppError> {
        match self.read() {
            Ok(state) => Ok(state.cache_get_string(key, default)),
            Err(e) => Err(SharedStateError(e.to_string())),
        }
    }
}
