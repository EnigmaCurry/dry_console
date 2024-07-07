use std::sync::{Arc, RwLock};

use aper::{NeverConflict, StateMachine};
use axum::{
    routing::{get, MethodRouter},
    Router,
};

use crate::SharedState;

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
pub fn main() -> Router<SharedState> {
    Router::new().merge(router())
}

fn route(path: &str, method_router: MethodRouter<SharedState>) -> Router<SharedState> {
    test_route(super::TestModule::Counter, path, method_router)
}

fn router() -> Router<SharedState> {
    async fn handler() -> &'static str {
        "Hello, World!\n"
    }
    route("/", get(handler))
}
