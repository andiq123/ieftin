use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// A request the API cannot answer, with the reason shown to the caller.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct BadRequest(pub &'static str);

impl IntoResponse for BadRequest {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": self.0 }))).into_response()
    }
}
