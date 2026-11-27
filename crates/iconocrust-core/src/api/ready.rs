//! Readiness endpoint: GET /v1/ready

use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
}

/// Returns 200 when required dependencies are usable, 503 otherwise.
pub async fn get_ready() -> (StatusCode, Json<ReadyResponse>) {
    // Milestone 0: no external dependencies yet; always ready
    (StatusCode::OK, Json(ReadyResponse { status: "ready" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ready_returns_ok() {
        let (status, _) = get_ready().await;
        assert_eq!(status, StatusCode::OK);
    }
}
