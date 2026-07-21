//! HTTP surface for the qt-core engine: state, router, handlers, DTOs.

use std::sync::Arc;

use axum::{routing::get, Json, Router};
use serde_json::json;

use qt_core::Engine;

/// Shared, read-only application state: the loaded engine behind an Arc.
pub struct AppState {
    pub engine: Arc<Engine>,
}

/// Build the router from shared state. Kept separate from socket binding so
/// tests can exercise handlers in-process via `oneshot`.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
