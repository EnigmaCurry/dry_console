use axum::{
    extract::{FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{io, sync::PoisonError};
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
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("SharedState error: {0}")]
    SharedState(String),
    #[error("Io error: {0}")]
    Io(io::Error),
    #[error("SharedState error: {0}")]
    Json(serde_json::Error),
    #[error("StateMachineConflict: {0}")]
    StateMachineConflict(String),
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
            AppError::Internal(_error)
            | AppError::SharedState(_error)
            | AppError::StateMachineConflict(_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_string(),
            ),
            AppError::Io(_error) => (StatusCode::BAD_REQUEST, "Bad request: IO error".to_string()),
            AppError::Json(_error) => {
                (StatusCode::BAD_REQUEST, "JSON validation error".to_string())
            }
        };
        (status, AppJson(ErrorResponse { error: e, trace_id })).into_response()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> AppError {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => AppError::Io(err.into()),
            Category::Syntax | Category::Data | Category::Eof => AppError::Json(err),
        }
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(_err: PoisonError<T>) -> Self {
        Self::SharedState("SharedState poison error".to_string())
    }
}

impl From<aper::NeverConflict> for AppError {
    fn from(_err: aper::NeverConflict) -> Self {
        Self::StateMachineConflict("State machine conflict".to_string())
    }
}
