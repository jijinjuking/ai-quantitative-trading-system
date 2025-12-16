use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde_json::json;

use crate::state::AppState;

/// 健康检查
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let health_status = json!({
        "service": "trading-engine",
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "database": state.check_database_health().await,
        "redis": state.check_redis_health().await,
        "kafka": state.check_kafka_health().await,
    });

    Json(health_status)
}

/// Prometheus 指标
pub async fn metrics(State(state): State<AppState>) -> Response {
    match state.metrics.gather() {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(e) => {
            tracing::error!("Failed to gather metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to gather metrics").into_response()
        }
    }
}