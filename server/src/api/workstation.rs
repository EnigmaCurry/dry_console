use crate::{api::route, app_state::SharedState};
use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .merge(workstation_dependencies())
}

fn workstation() -> Router<SharedState> {
    async fn handler() -> Json<serde_json::Value> {
        let host = hostname::get().unwrap_or_else(|_| "Unknown".into());
        let uid = users::get_current_uid();
        let user = users::get_user_by_uid(users::get_current_uid()).unwrap();
        let username = user.name().to_string_lossy();
        Json(json!({ "workstation": {
            "hostname": host.to_string_lossy(),
            "user": {"uid":uid,"name":username},
        } }))
    }
    route("/", get(handler))
}

fn workstation_dependencies() -> Router<SharedState> {
    async fn handler() -> &'static str {
        "Workstation foo"
    }
    route("/dependencies", get(handler))
}
