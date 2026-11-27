mod health;

use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/v1/health", get(health::get_health))
}
