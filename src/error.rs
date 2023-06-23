use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    Internal(anyhow::Error),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}
