use axum::body::Bytes;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

////////////////////////////////////////////////////////////////////////////////
// Global app state
////////////////////////////////////////////////////////////////////////////////
pub trait AppState {
    fn cache_set(&self, key: &str, value: &Bytes);
    fn cache_set_string(&self, key: &str, value: &str);
    fn cache_get(&self, key: &str, default: &Bytes) -> Bytes;
    fn cache_get_string(&self, key: &str, default: &str) -> String;
    fn cache_set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String>;
    fn cache_get_json<T: DeserializeOwned + Clone>(
        &self,
        key: &str,
        default: &T,
    ) -> Result<T, String>;
}
impl AppState for SharedState {
    fn cache_set(&self, key: &str, value: &Bytes) {
        self.insert(key.to_string(), value.clone());
    }
    fn cache_set_string(&self, key: &str, value: &str) {
        self.cache_set(key, &Bytes::from(value.to_string()));
    }
    fn cache_get(&self, key: &str, default: &Bytes) -> Bytes {
        match self.get(key) {
            Some(d) => d.value().clone(),
            None => Bytes::from(default.clone()),
        }
    }
    fn cache_get_string(&self, key: &str, default: &str) -> String {
        std::str::from_utf8(&self.cache_get(key, &Bytes::from(default.to_string())))
            .unwrap_or(&default)
            .to_string()
    }
    fn cache_set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        match serde_json::to_string(value) {
            Ok(json_string) => Ok(self.cache_set_string(key, &json_string)),
            Err(e) => Err(e.to_string()),
        }
    }

    fn cache_get_json<T: DeserializeOwned + Clone>(
        &self,
        key: &str,
        default: &T,
    ) -> Result<T, String> {
        let json_string = self.cache_get_string(key, "");
        if json_string.is_empty() {
            return Ok(default.clone());
        }
        match serde_json::from_str(&json_string) {
            Ok(value) => Ok(value),
            Err(e) => Err(e.to_string()),
        }
    }
}
pub type SharedState = Arc<Mutex<DashMap<String, String>>>;
