//! Control-plane HTTP service.
//!
//! Phase 1 currently implements the device registration slice: a small Axum
//! application with health probes and an idempotent registration endpoint
//! backed by an in-memory device registry. The registry is intentionally
//! storage-agnostic so the PostgreSQL-backed store can replace it later without
//! changing the HTTP surface.

mod registry;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use edgefleet_types::DeviceRegistrationRequest;
use serde::Serialize;

pub use registry::{DeviceRecord, DeviceRegistry, RegistrationError};

/// Shared application state handed to each request handler.
#[derive(Clone, Default)]
pub struct AppState {
    pub registry: DeviceRegistry,
}

/// Build the control-plane router with a fresh in-memory registry.
pub fn app() -> Router {
    app_with_state(AppState::default())
}

/// Build the control-plane router around an existing [`AppState`].
///
/// Useful for tests that want to inspect the registry after driving requests.
pub fn app_with_state(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/devices/register", post(register))
        .with_state(state)
}

/// Serve the control-plane application on an already-bound listener.
pub async fn serve(listener: tokio::net::TcpListener) -> std::io::Result<()> {
    axum::serve(listener, app()).await
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> &'static str {
    "ready"
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<DeviceRegistrationRequest>,
) -> Response {
    match state.registry.register(&request, Utc::now()) {
        Ok(response) => {
            tracing::info!(device_id = %response.device_id, "device registered");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(RegistrationError::EmptyField { field }) => {
            tracing::warn!(field, "rejected registration with empty field");
            let body = ErrorBody {
                error: format!("{field} must not be empty"),
            };
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}
