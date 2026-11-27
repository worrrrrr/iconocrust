//! API error contract: consistent JSON error envelope.

use axum::{
    Json,
    body::HttpBody,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use super::REQUEST_ID_HEADER;

/// Error envelope matching CORE_SPEC §6.5.
#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Typed API error enum.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal error")]
    Internal,
    #[error("service unavailable: {0}")]
    Unavailable(String),
}

impl ApiError {
    /// Stable machine-readable error code (CORE_SPEC §6.5).
    pub fn code(&self) -> &'static str {
        match self {
            ApiError::Validation(_) => "validation_error",
            ApiError::NotFound(_) => "not_found",
            ApiError::Internal => "internal_error",
            ApiError::Unavailable(_) => "service_unavailable",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::Validation(m) => m.clone(),
            ApiError::NotFound(m) => m.clone(),
            ApiError::Internal => "An unexpected error occurred.".to_string(),
            ApiError::Unavailable(m) => m.clone(),
        }
    }
    /// Serialize this error into the API envelope, using the request
    /// correlation ID from middleware extensions when available.
    pub fn to_envelope(&self, request_id: &str) -> ErrorEnvelope {
        ErrorEnvelope {
            error: ErrorDetail {
                code: self.code().to_string(),
                message: self.message(),
                request_id: request_id.to_string(),
                details: None,
            },
        }
    }
}

/// Fallback `IntoResponse`: keeps the generated UUID correlation ID so an
/// error rendered outside the router still carries a usable identifier.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4().to_string();
        (self.status(), Json(self.to_envelope(&request_id))).into_response()
    }
}

/// Router middleware that renders error responses with the request's
/// `x-request-id`, so error bodies correlate with server logs (CORE_SPEC §6.5).
///
/// The request ID comes from the `RequestId` extension installed by
/// `SetRequestIdLayer`, which already honors a client-supplied header.
pub async fn error_handling_middleware(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let response = next.run(request).await;

    if response.status().is_success() || response.status().is_redirection() {
        return response;
    }

    // The request ID from an outer layer, if one was installed.
    let responded_id = response
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or(&request_id)
        .to_string();

    if response.status().is_server_error() {
        tracing::error!(
            request_id = %responded_id,
            status = response.status().as_u16(),
            "request failed"
        );
    } else {
        tracing::debug!(
            request_id = %responded_id,
            status = response.status().as_u16(),
            "request rejected"
        );
    }

    // Fallback 404/405 responses carry no body; give them the standard
    // envelope so every error path is shaped the same way for clients.
    if response.body().size_hint().exact() == Some(0) {
        let (status, message) = if status_is_not_found(response.status()) {
            (StatusCode::NOT_FOUND, "No route matches this request.")
        } else if status_is_method_not_allowed(response.status()) {
            (
                StatusCode::METHOD_NOT_ALLOWED,
                "This method is not supported for this route.",
            )
        } else {
            (response.status(), "The request could not be completed.")
        };

        let code = match status {
            StatusCode::NOT_FOUND => "not_found",
            StatusCode::METHOD_NOT_ALLOWED => "method_not_allowed",
            _ => "error",
        };

        let envelope = ErrorEnvelope {
            error: ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                request_id: responded_id,
                details: None,
            },
        };

        let mut rebuilt = (status, Json(envelope)).into_response();
        // Preserve any headers the inner response carried (for example
        // `allow` on a 405); the request ID layer still runs afterwards.
        for (name, value) in response.headers() {
            rebuilt.headers_mut().insert(name, value.clone());
        }
        return rebuilt;
    }

    response
}

fn status_is_not_found(status: StatusCode) -> bool {
    status == StatusCode::NOT_FOUND
}

fn status_is_method_not_allowed(status: StatusCode) -> bool {
    status == StatusCode::METHOD_NOT_ALLOWED
}

/// Helper to build an error envelope with a specific request ID.
#[allow(dead_code)]
pub fn error_envelope(code: &str, message: &str, request_id: &str) -> ErrorEnvelope {
    ErrorEnvelope {
        error: ErrorDetail {
            code: code.to_string(),
            message: message.to_string(),
            request_id: request_id.to_string(),
            details: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_is_stable() {
        let err = ApiError::Validation("bad input".to_string());
        assert_eq!(err.code(), "validation_error");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn internal_error_hides_details() {
        let err = ApiError::Internal;
        assert_eq!(err.message(), "An unexpected error occurred.");
        assert!(!err.message().contains("stack"));
    }

    #[test]
    fn error_envelope_serializes() {
        let env = error_envelope("test_code", "test msg", "req-123");
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("test_code"));
        assert!(json.contains("req-123"));
    }

    #[test]
    fn to_envelope_uses_supplied_request_id() {
        let env = ApiError::Validation("bad".to_string()).to_envelope("req-abc");
        assert_eq!(env.error.request_id, "req-abc");
        assert_eq!(env.error.code, "validation_error");
    }
}
