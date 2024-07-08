use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;
use thiserror;
use ulid::Ulid;

/// JSON response
pub type JsonResult<T> = Result<AppJson<T>, AppError>;
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

/// Error response
// https://docs.rs/thiserror/latest/thiserror/
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("InternalError: {0}")]
    InternalError(String),
    #[error("SharedStateError: {0}")]
    SharedStateError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
            trace_id: Ulid,
        }
        let trace_id = Ulid::new();
        tracing::debug!("Error trace_id: {}", trace_id);
        let (status, e) = match self {
            AppError::InternalError(error) | AppError::SharedStateError(error) => {
                tracing::error!(%error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                )
            }
        };
        (status, AppJson(ErrorResponse { error: e, trace_id })).into_response()
    }
}
