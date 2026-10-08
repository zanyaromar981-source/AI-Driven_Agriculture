use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use tokio::time::timeout;

use crate::shared::AppState;

const PING_TIMEOUT: Duration = Duration::from_secs(5);

#[utoipa::path(
    get,
    path = "/status",
    responses((status = 200, description = "Process is running")),
    tag = "farm-doctor-api"
)]
pub async fn liveness(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "startTime": state.startup_time,
        "host": hostname(),
    }))
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Database ping succeeded"),
        (status = 503, description = "Database ping failed")
    ),
    tag = "farm-doctor-api"
)]
pub async fn readiness(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    readiness_response(database_ping(&state).await, unix_now())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(liveness))
        .route("/health", get(readiness))
}

pub(crate) fn readiness_response(database_ok: bool, timestamp: u64) -> (StatusCode, Json<Value>) {
    let (code, status, message) = if database_ok {
        (StatusCode::OK, "healthy", "Database connection OK")
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "unhealthy",
            "Database unreachable",
        )
    };

    (
        code,
        Json(json!({
            "status": status,
            "timestamp": timestamp,
            "checks": {
                "database": {
                    "status": status,
                    "message": message,
                }
            }
        })),
    )
}

async fn database_ping(state: &AppState) -> bool {
    match timeout(PING_TIMEOUT, state.database.execute_unprepared("SELECT 1")).await {
        Ok(Ok(_)) => true,
        Ok(Err(error)) => {
            tracing::warn!(%error, "readiness database ping failed");
            false
        }
        Err(_) => {
            tracing::warn!(
                timeout_secs = PING_TIMEOUT.as_secs(),
                "readiness database ping timed out"
            );
            false
        }
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| {
        std::fs::read_to_string("/proc/sys/kernel/hostname")
            .map(|name| name.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    })
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reachable_database_is_ready() {
        let (code, Json(body)) = readiness_response(true, 42);

        assert_eq!(code, StatusCode::OK);
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["timestamp"], 42);
        assert_eq!(body["checks"]["database"]["status"], "healthy");
        assert_eq!(
            body["checks"]["database"]["message"],
            "Database connection OK"
        );
    }

    #[test]
    fn an_unreachable_database_still_returns_its_json() {
        let (code, Json(body)) = readiness_response(false, 42);

        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["status"], "unhealthy");
        assert_eq!(body["checks"]["database"]["status"], "unhealthy");
    }

    #[test]
    fn a_failure_never_reports_the_underlying_database_error() {
        let (_, Json(body)) = readiness_response(false, 42);

        assert_eq!(
            body["checks"]["database"]["message"],
            "Database unreachable"
        );
    }

    #[test]
    fn health_routes_can_be_built_without_a_live_database() {
        let _ = routes();
    }
}
