use std::sync::RwLockWriteGuard;

use aper::{NeverConflict, StateMachine};
use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use serde_json;

use crate::{
    app_state::{AppState, SharedState},
    response::{AppError, AppJson, JsonResult},
    AppMethodRouter, AppRouter,
};

use super::test_route;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Counter
////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Counter {
    value: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum CounterTransition {
    Add(i64),
    Subtract(i64),
    Reset,
}
impl StateMachine for Counter {
    type Transition = CounterTransition;
    type Conflict = NeverConflict;
    fn apply(&self, event: &CounterTransition) -> Result<Counter, NeverConflict> {
        match event {
            CounterTransition::Add(i) => Ok(Counter {
                value: self.value + i,
            }),
            CounterTransition::Subtract(i) => Ok(Counter {
                value: self.value - i,
            }),
            CounterTransition::Reset => Ok(Counter { value: 0 }),
        }
    }
}
#[allow(dead_code)]
impl Counter {
    pub fn add(&self, i: i64) -> CounterTransition {
        CounterTransition::Add(i)
    }
    pub fn subtract(&self, i: i64) -> CounterTransition {
        CounterTransition::Subtract(i)
    }
    pub fn reset(&self) -> CounterTransition {
        CounterTransition::Reset
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
    async fn handler(State(state): State<SharedState>) -> JsonResult<Counter> {
        match state.read() {
            Ok(state) => match state.cache_get_string("test::counter", "").as_str() {
                "" => match serde_json::to_string(&Counter::default()) {
                    Ok(c) => Ok(AppJson(serde_json::from_str(&c)?)),
                    Err(e) => Err(AppError::Internal(e.to_string())),
                },
                j => Ok(AppJson(serde_json::from_str(j)?)),
            },
            Err(e) => Err(AppError::SharedState(e.to_string())),
        }
    }
    route("/", get(handler))
}

fn update_counter() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<Counter> {
        fn from_json(c: &str) -> Result<Counter, serde_json::Error> {
            serde_json::from_str(c)
        }
        fn to_json(c: &Counter) -> Result<String, serde_json::Error> {
            serde_json::to_string(&c)
        }
        fn get_counter(
            state: &RwLockWriteGuard<'_, AppState>,
        ) -> Result<Counter, serde_json::Error> {
            match state.cache_get_string("test::counter", "").as_str() {
                "" => Ok(Counter::default()),
                j => Ok(from_json(j)?),
            }
        }
        let mut state = state.write()?;
        let c = get_counter(&state)?;
        let c = c.apply(&c.add(1))?;
        let j = to_json(&c)?;
        state.cache_set_string("test::counter", &j);
        Ok(AppJson(c))
    }
    route("/", post(handler))
}
