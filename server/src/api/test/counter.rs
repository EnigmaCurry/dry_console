use std::{convert::Infallible, sync::RwLockWriteGuard};

use aper::{NeverConflict, StateMachine};
use axum::{
    extract::State,
    routing::{get, post, MethodRouter},
    Router,
};
use serde_json;

use super::test_route;
use crate::{
    app_state::{AppState, SharedState},
    response::{AppError, AppJson, JsonResult},
    AppRouter,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

////////////////////////////////////////////////////////////////////////////////
// Counter
////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
pub struct TestCounter {
    value: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CounterTransition {
    Add(i64),
    Subtract(i64),
    Reset,
}
impl StateMachine for TestCounter {
    type Transition = CounterTransition;
    type Conflict = NeverConflict;
    fn apply(&self, event: &CounterTransition) -> Result<TestCounter, NeverConflict> {
        match event {
            CounterTransition::Add(i) => Ok(TestCounter {
                value: self.value + i,
            }),
            CounterTransition::Subtract(i) => Ok(TestCounter {
                value: self.value - i,
            }),
            CounterTransition::Reset => Ok(TestCounter { value: 0 }),
        }
    }
}
#[allow(dead_code)]
impl TestCounter {
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

fn route(path: &str, method_router: MethodRouter<SharedState, Infallible>) -> AppRouter {
    test_route(super::TestModule::Counter, path, method_router)
}

#[utoipa::path(
    get,
    path = "/api/test/counter/",
    responses(
        (status = OK, description = "Get counter value", body = TestCounter)
    )
)]
fn get_counter() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<TestCounter> {
        match state.read() {
            Ok(state) => match state.cache_get_string("test::counter", "").as_str() {
                "" => match serde_json::to_string(&TestCounter::default()) {
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

#[utoipa::path(
    post,
    path = "/api/test/counter/",
    responses(
        (status = OK, description = "Increment counter value", body = TestCounter)
    ),
    security(
        ("bearer_token" = [])
    )
)]
fn update_counter() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<TestCounter> {
        fn from_json(c: &str) -> Result<TestCounter, serde_json::Error> {
            serde_json::from_str(c)
        }
        fn to_json(c: &TestCounter) -> Result<String, serde_json::Error> {
            serde_json::to_string(&c)
        }
        fn get_counter(
            state: &RwLockWriteGuard<'_, AppState>,
        ) -> Result<TestCounter, serde_json::Error> {
            match state.cache_get_string("test::counter", "").as_str() {
                "" => Ok(TestCounter::default()),
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
