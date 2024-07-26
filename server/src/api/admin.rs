use std::sync::Arc;

use crate::{api::route, app_state::SharedState};
use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Router};
use tokio::sync::{oneshot, Mutex};

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(shutdown())
        .with_state(SharedState::default())
}

#[utoipa::path(
    post,
    path = "/api/admin/shutdown/",
    responses(
        (status = OK, description = "Shutdown service", body = str)
    )
)]
fn shutdown() -> Router<SharedState> {
    async fn handler(
        Extension(shutdown_tx): Extension<Arc<Mutex<Option<oneshot::Sender<()>>>>>,
    ) -> impl IntoResponse {
        if let Some(shutdown_tx) = shutdown_tx.lock().await.take() {
            let _ = shutdown_tx.send(());
            (StatusCode::OK, "Server is shutting down")
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server is already shutting down",
            )
        }
    }
    route("/shutdown", post(handler))
}
