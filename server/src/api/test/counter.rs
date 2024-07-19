use super::test_route;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use aper::{NeverConflict, StateMachine};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use dashmap::DashMap;
use serde_json;
use tokio::sync::Mutex;

use crate::{
    app_state::{AppState, SharedState},
    response::{AppJson, JsonResult},
    AppMethodRouter, AppRouter,
};

use serde::{Deserialize, Serialize};
use tracing;

////////////////////////////////////////////////////////////////////////////////
// Counter
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Default)]
struct AtomicCounter(AtomicUsize);

// Implement Serialize for AtomicCounter
impl Serialize for AtomicCounter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.load() as u64)
    }
}

// Implement Deserialize for AtomicCounter
impl<'de> Deserialize<'de> for AtomicCounter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = u64::deserialize(deserializer)? as usize;
        Ok(AtomicCounter::new(val))
    }
}

impl AtomicCounter {
    fn new(val: usize) -> Self {
        AtomicCounter(AtomicUsize::new(val))
    }

    fn atomic_add(&self, val: usize) -> usize {
        self.0.fetch_add(val, Ordering::SeqCst);
        self.load()
    }

    fn atomic_sub(&self, val: usize) -> usize {
        self.0.fetch_sub(val, Ordering::SeqCst);
        self.load()
    }

    fn load(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

impl Clone for AtomicCounter {
    fn clone(&self) -> Self {
        AtomicCounter::new(self.load())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum CounterTransition {
    Add(usize),
    Subtract(usize),
    Reset,
}

impl StateMachine for AtomicCounter {
    type Transition = CounterTransition;
    type Conflict = NeverConflict;

    fn apply(&self, event: &CounterTransition) -> Result<AtomicCounter, NeverConflict> {
        match event {
            CounterTransition::Add(i) => Ok(AtomicCounter::new(self.atomic_add(*i))),
            CounterTransition::Subtract(i) => Ok(AtomicCounter::new(self.atomic_sub(*i))),
            CounterTransition::Reset => Ok(AtomicCounter::new(0)),
        }
    }
}

#[allow(dead_code)]
impl AtomicCounter {
    pub fn add(&self, i: usize) -> CounterTransition {
        CounterTransition::Add(i)
    }

    pub fn subtract(&self, i: usize) -> CounterTransition {
        CounterTransition::Subtract(i)
    }

    pub fn reset(&self) -> CounterTransition {
        CounterTransition::Reset
    }
}

////////////////////////////////////////////////////////////////////////////////
// SharedState
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct SharedState {
    map: Arc<Mutex<DashMap<String, String>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(DashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let map = self.map.lock().await;
        map.get(key).map(|v| v.value().clone())
    }

    pub async fn set(&self, key: &str, value: String) {
        let mut map = self.map.lock().await;
        map.insert(key.to_string(), value);
    }
}

////////////////////////////////////////////////////////////////////////////////
// Routes:
////////////////////////////////////////////////////////////////////////////////

pub fn main() -> AppRouter {
    Router::new().merge(get_counter()).merge(update_counter())
}

fn route(path: &str, method_router: AppMethodRouter) -> AppRouter {
    test_route(super::TestModule::Counter, path, method_router)
}

fn get_counter() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<AtomicCounter> {
        let counter_str = state
            .get("test::counter")
            .await
            .unwrap_or_else(|| "0".to_string());
        let c = serde_json::from_str(&counter_str).unwrap_or_default();
        Ok(AppJson(c))
    }
    route("/", get(handler))
}

fn update_counter() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<AtomicCounter> {
        let counter_str = state
            .get("test::counter")
            .await
            .unwrap_or_else(|| "0".to_string());
        let c = serde_json::from_str(&counter_str).unwrap_or_default();
        let c2 = c.apply(&c.add(1))?;
        let counter_json = serde_json::to_string(&c2)?;
        state.set("test::counter", counter_json).await;
        Ok(AppJson(c2))
    }
    route("/", post(handler))
}
