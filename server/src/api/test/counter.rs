use std::sync::{Arc, RwLock};

use aper::{NeverConflict, StateMachine};
use axum::{
    routing::{get, MethodRouter},
    Router,
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
// Global app state
////////////////////////////////////////////////////////////////////////////////
type SharedState = Arc<RwLock<AppState>>;
#[derive(Default)]
struct AppState {
    counter: Counter,
}

////////////////////////////////////////////////////////////////////////////////
// Routes:
////////////////////////////////////////////////////////////////////////////////
pub fn main() -> Router {
    let shared_state = SharedState::default();

    Router::new()
        .merge(router())
        .with_state(Arc::clone(&shared_state))
}

fn route(path: &str, method_router: MethodRouter<()>) -> Router {
    test_route(super::TestModule::Counter, path, method_router)
}

fn router() -> Router {
    async fn handler() -> &'static str {
        "Hello, World!\n"
    }
    route("/", get(handler))
}
