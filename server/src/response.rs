use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;
use thiserror;

/// JSON response
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
    #[error("SharedStateError: {0}")]
    SharedStateError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let (status, message) = match self {
            AppError::SharedStateError(e) => {
                tracing::error!(%e, "error with shared state");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
        };
        (status, AppJson(ErrorResponse { message })).into_response()
    }
}
