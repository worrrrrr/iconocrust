//! Router-level HTTP tests for the Core API surface (CORE_SPEC §13).
//!
//! These exercise the real `router()` so layer wiring — request ID
//! set/propagate order, error envelopes — is covered end to end.

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use iconocrust_core::api::{ApiError, REQUEST_ID_HEADER, router};
use tower::ServiceExt;

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).expect("response body must be JSON")
}

fn request_id(response: &Response) -> Option<&str> {
    response
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
}

#[tokio::test]
async fn health_returns_ok_json() {
    let response = router()
        .oneshot(Request::get("/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE),
        Some(&"application/json".parse().unwrap())
    );

    let json = body_json(response).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "iconocrust-core");
}

#[tokio::test]
async fn ready_returns_ok_json() {
    let response = router()
        .oneshot(Request::get("/v1/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["status"], "ready");
}

#[tokio::test]
async fn generated_request_id_is_uuid_and_propagated_on_success() {
    let response = router()
        .oneshot(Request::get("/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let id = request_id(&response).expect("x-request-id must be set on success");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(uuid::Uuid::parse_str(id).is_ok(), "expected a UUID v4");
}

#[tokio::test]
async fn client_request_id_is_echoed_on_success() {
    let response = router()
        .oneshot(
            Request::get("/v1/ready")
                .header(REQUEST_ID_HEADER, "req-client-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(request_id(&response), Some("req-client-123"));
}

#[tokio::test]
async fn unknown_route_returns_envelope_correlated_to_request_id() {
    let response = router()
        .oneshot(
            Request::get("/v1/does-not-exist")
                .header(REQUEST_ID_HEADER, "req-404")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(request_id(&response), Some("req-404"));

    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "not_found");
    assert_eq!(json["error"]["request_id"], "req-404");
}

#[tokio::test]
async fn wrong_method_returns_405_envelope() {
    let response = router()
        .oneshot(Request::post("/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        request_id(&response).is_some(),
        "x-request-id must survive the 405 rebuild"
    );

    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "method_not_allowed");
    assert!(json["error"]["request_id"].is_string());
}

#[tokio::test]
async fn api_error_response_matches_envelope_contract() {
    let response = ApiError::Validation("field `name` is required".to_string()).into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "validation_error");
    assert_eq!(json["error"]["message"], "field `name` is required");
    assert!(
        uuid::Uuid::parse_str(json["error"]["request_id"].as_str().unwrap()).is_ok(),
        "fallback request id must be a usable identifier"
    );
    assert!(json["error"].get("details").is_none());
}

#[tokio::test]
async fn internal_error_response_leaks_nothing() {
    let response = ApiError::Internal.into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "internal_error");
    assert_eq!(json["error"]["message"], "An unexpected error occurred.");
}

#[tokio::test]
async fn unavailable_maps_to_503() {
    let response = ApiError::Unavailable("dependency offline".to_string()).into_response();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "service_unavailable");
}
