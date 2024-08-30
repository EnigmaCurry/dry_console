use crate::api::auth::TOKEN_CACHE_NAME;
use crate::api::token::generate_token;
use crate::api::workstation::platform::detect_platform;
use crate::api::workstation::WorkstationDependencyState;
use crate::response::AppError;
use crate::Opt;
use axum::body::Bytes;
use dry_console_dto::workstation::Platform;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

////////////////////////////////////////////////////////////////////////////////
// Global app state
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug)]
pub struct AppState {
    #[allow(dead_code)]
    pub opt: Opt,
    pub cache: HashMap<String, Bytes>,
    pub login_allowed: bool,
    pub sudo_enabled: bool,
    pub missing_dependencies: Vec<WorkstationDependencyState>,
    pub platform: Platform,
    pub command_library_overlay: HashMap<String, String>,
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

pub fn create_shared_state(opt: &Opt) -> SharedState {
    let token = generate_token();
    let url = format!("http://{0}:{1}/login#token:{token}", opt.addr, opt.port);
    info!("\n\nLogin URL:\n{0}\n", url);

    Arc::new(RwLock::new(AppState {
        opt: opt.clone(),
        cache: HashMap::from([(TOKEN_CACHE_NAME.to_string(), Bytes::from(token))]),
        login_allowed: true,
        sudo_enabled: false,
        missing_dependencies: Vec::<WorkstationDependencyState>::new(),
        command_library_overlay: HashMap::<String, String>::new(),
        platform: detect_platform(),
    }))
}

pub trait ShareableState {
    #[allow(dead_code)]
    async fn cache_set(&mut self, key: &str, value: &Bytes) -> Result<(), AppError>;
    #[allow(dead_code)]
    async fn cache_set_string(&mut self, key: &str, value: &str) -> Result<(), AppError>;
    #[allow(dead_code)]
    async fn cache_get(&self, key: &str, default: &Bytes) -> Bytes;
    #[allow(dead_code)]
    async fn cache_get_string(&self, key: &str, default: &str) -> String;
}
impl ShareableState for SharedState {
    async fn cache_set(&mut self, key: &str, value: &Bytes) -> Result<(), AppError> {
        // Locks the entire hashmap to do a single atomic write:
        let mut state = self.write().await;
        state.cache_set(key, value);
        Ok(())
    }

    async fn cache_set_string(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        //Locks the entire hashmap to do a single atomic write:
        let mut state = self.write().await;
        state.cache_set_string(key, value);
        Ok(())
    }

    async fn cache_get(&self, key: &str, default: &Bytes) -> Bytes {
        //Block only if there is an atomic write in progress,
        //otherwise, multiple readers can read at the same time:
        let state = self.read().await;
        state.cache_get(key, default)
    }

    async fn cache_get_string(&self, key: &str, default: &str) -> String {
        //Block only if there is an atomic write in progress,
        //otherwise, multiple readers can read at the same time:
        let state = self.read().await;
        state.cache_get_string(key, default)
    }
}
