use aper::{NeverConflict, StateMachine};
use axum::{extract::State, routing::get, Router};
use serde_json;

use crate::{
    app_state::SharedState,
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
    Router::new().merge(router())
}

fn route(path: &str, method_router: AppMethodRouter) -> AppRouter {
    test_route(super::TestModule::Counter, path, method_router)
}

fn router() -> AppRouter {
    async fn handler(State(mut state): State<SharedState>) -> JsonResult<Counter> {
        match state.read() {
            Ok(state) => match state.cache_get_string("test::counter", "").as_str() {
                "" => match serde_json::to_string(&Counter::default()) {
                    Ok(c) => Ok(AppJson(serde_json::from_str(&c)?)),
                    Err(e) => Err(AppError::Internal(e.to_string())),
                },
                j => {
                    todo!();
                }
            },
            Err(e) => Err(AppError::SharedState(e.to_string())),
        }

        // match state.cache_get_string("test::counter", "") {
        //     Ok(s) => {
        //         match s {
        //             s if s.is_empty() => {
        //                 c = Counter::default();
        //             }
        //             s => {
        //                 c = serde_json::from_str(&s).unwrap();
        //             }
        //         };
        //         c = c.apply(&c.add(1)).unwrap();
        //         match serde_json::to_string(&c) {
        //             Ok(j) => match state.cache_set_string("test::counter", &j) {
        //                 Ok(_) => Ok(AppJson(c)),
        //                 Err(e) => Err(e),
        //             },
        //             Err(e) => Err(AppError::InternalError(e.to_string())),
        //         }
        //     }
        //     Err(e) => Err(e),
        // }
    }
    route("/", get(handler))
}
