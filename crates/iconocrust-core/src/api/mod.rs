mod error;
mod health;
mod ready;

use axum::{Router, http::HeaderName, middleware, routing::get};
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use uuid::Uuid;

pub use error::{ApiError, ErrorDetail, ErrorEnvelope};
pub use health::get_health;
pub use ready::get_ready;

/// Header carrying the request correlation ID (CORE_SPEC §6.5).
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Simple request ID generator using UUID v4.
///
/// Honors a client-supplied `x-request-id` when present and non-empty,
/// otherwise mints a fresh UUID so responses and logs stay correlated.
#[derive(Clone, Default)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, request: &axum::http::Request<B>) -> Option<RequestId> {
        if let Some(incoming) = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .filter(|v| !v.trim().is_empty())
            .and_then(|v| v.parse().ok())
        {
            return Some(RequestId::new(incoming));
        }

        let id = Uuid::new_v4().to_string();
        Some(RequestId::new(id.parse().ok()?))
    }
}

/// Build the Core HTTP router.
pub fn router() -> Router {
    let request_id_header = HeaderName::from_static(REQUEST_ID_HEADER);

    Router::new()
        .route("/v1/health", get(health::get_health))
        .route("/v1/ready", get(ready::get_ready))
        .layer(middleware::from_fn(error::error_handling_middleware))
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::IntoResponse,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_returns_ok_and_service_name() {
        let response = router()
            .oneshot(Request::get("/v1/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(axum::http::header::CONTENT_TYPE),
            Some(&"application/json".parse().unwrap())
        );
        let json = body_json(response).await;
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "iconocrust-core");
    }

    #[tokio::test]
    async fn ready_returns_ok() {
        let response = router()
            .oneshot(Request::get("/v1/ready").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["status"], "ready");
    }

    #[tokio::test]
    async fn unknown_route_returns_404_with_envelope() {
        let response = router()
            .oneshot(
                Request::get("/v1/nope")
                    .header(REQUEST_ID_HEADER, "req-404-check")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get(REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok()),
            Some("req-404-check")
        );

        let json = body_json(response).await;
        assert_eq!(json["error"]["code"], "not_found");
        assert_eq!(json["error"]["request_id"], "req-404-check");
    }

    #[tokio::test]
    async fn wrong_method_returns_405() {
        let response = router()
            .oneshot(Request::post("/v1/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        let json = body_json(response).await;
        assert_eq!(json["error"]["code"], "method_not_allowed");
        assert!(json["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn request_id_is_generated_and_propagated() {
        let response = router()
            .oneshot(Request::get("/v1/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let id = response
            .headers()
            .get(REQUEST_ID_HEADER)
            .expect("x-request-id must be propagated")
            .to_str()
            .unwrap()
            .to_string();

        assert!(!id.is_empty());
        assert!(uuid::Uuid::parse_str(&id).is_ok(), "expected a UUID v4");
    }

    #[tokio::test]
    async fn client_request_id_is_preserved() {
        let response = router()
            .oneshot(
                Request::get("/v1/ready")
                    .header(REQUEST_ID_HEADER, "req-client-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response
                .headers()
                .get(REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok()),
            Some("req-client-123")
        );
    }

    #[tokio::test]
    async fn error_response_matches_envelope() {
        let response = ApiError::NotFound("no such resource".to_string()).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let json = body_json(response).await;
        assert_eq!(json["error"]["code"], "not_found");
        assert_eq!(json["error"]["message"], "no such resource");
        assert!(json["error"]["request_id"].is_string());
        assert!(json["error"].get("details").is_none());
    }

    #[tokio::test]
    async fn internal_error_hides_details_in_response() {
        let response = ApiError::Internal.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = body_json(response).await;
        assert_eq!(json["error"]["code"], "internal_error");
        assert_eq!(json["error"]["message"], "An unexpected error occurred.");
    }
}
