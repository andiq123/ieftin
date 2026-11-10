use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Why the API would not answer a request, with what the caller is told.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(&'static str),
    #[error("too many searches")]
    TooManyRequests { retry_after_seconds: u64 },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({ "error": self.to_string() }));
        match self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, body).into_response(),
            Self::TooManyRequests { retry_after_seconds } => {
                let retry_after = [(header::RETRY_AFTER, retry_after_seconds.to_string())];
                (StatusCode::TOO_MANY_REQUESTS, retry_after, body).into_response()
            }
        }
    }
}
