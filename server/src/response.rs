use anyhow::{anyhow, Error};
use axum::{
    extract::FromRequest,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{io, sync::PoisonError};
use tracing::error;
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

/// Error response captures error and optionally the request URL
// https://docs.rs/thiserror/latest/thiserror/
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(Error, Option<String>),
    #[error("SharedState error: {0}")]
    SharedState(Error, Option<String>),
    #[error("Io error: {0}")]
    Io(io::Error, Option<String>),
    #[error("JSON validation error: {0}")]
    Json(serde_json::Error, Option<String>),
    #[error("StateMachineConflict: {0}")]
    StateMachineConflict(Error, Option<String>),
    #[error("Not found")]
    NotFound(Option<String>),
    #[error("Config error: {0}")]
    Config(String, Option<String>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
            trace_id: Ulid,
            request_url: Option<String>, // Include the request URL in the response
        }

        let trace_id = Ulid::new();
        tracing::debug!("Error trace_id: {}", trace_id);

        let (status, e, url) = match &self {
            AppError::Internal(error, url)
            | AppError::SharedState(error, url)
            | AppError::StateMachineConflict(error, url) => {
                error!(
                    "Internal server error: {:?} - Request URL: {:?}",
                    error,
                    url.clone().unwrap_or("None".to_string())
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                    url.clone(),
                )
            }
            AppError::Io(error, url) => {
                error!(
                    "IO error: {:?} - Request URL: {:?}",
                    error,
                    url.clone().unwrap_or("None".to_string())
                );
                (
                    StatusCode::BAD_REQUEST,
                    "Bad request: IO error".to_string(),
                    url.clone(),
                )
            }
            AppError::Config(error, url) => {
                error!(
                    "Configuration error: {:?} - Request URL: {:?}",
                    error,
                    url.clone().unwrap_or("None".to_string())
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                    url.clone(),
                )
            }
            AppError::Json(error, url) => {
                error!(
                    "JSON validation error: {:?} - Request URL: {:?}",
                    error,
                    url.clone().unwrap_or("None".to_string())
                );
                (
                    StatusCode::BAD_REQUEST,
                    "JSON validation error".to_string(),
                    url.clone(),
                )
            }
            AppError::NotFound(url) => (
                StatusCode::NOT_FOUND,
                "Object not found".to_string(),
                url.clone(),
            ),
        };

        (
            status,
            AppJson(ErrorResponse {
                error: e,
                trace_id,
                request_url: url,
            }),
        )
            .into_response()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> AppError {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => AppError::Io(err.into(), None), // Pass URL here if available
            Category::Syntax | Category::Data | Category::Eof => AppError::Json(err, None),
        }
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(_err: PoisonError<T>) -> Self {
        Self::SharedState(anyhow!("SharedState poison error"), None) // Pass URL here if available
    }
}

impl From<aper::NeverConflict> for AppError {
    fn from(_err: aper::NeverConflict) -> Self {
        Self::StateMachineConflict(anyhow!("State machine conflict"), None) // Pass URL here if available
    }
}
