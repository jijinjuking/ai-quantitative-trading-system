use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use shared_protocols::http::ApiResponse;

use crate::state::AppState;

/// 健康检查处理器
pub async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_status = state.health_check().await;
    
    let response = json!({
        "status": health_status.status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "service": "gateway",
        "checks": health_status.checks
    });

    Ok(Json(response))
}

/// 列出所有服务状态
pub async fn list_services(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let services = state.service_registry.get_all_services().await;
    let stats = state.service_registry.get_service_stats().await;
    
    let response = json!({
        "services": services,
        "stats": stats,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(response))
}

/// 熔断器状态
pub async fn circuit_breaker_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let breakers = state.circuit_breakers.read().await;
    let mut status = serde_json::Map::new();
    
    for (service, breaker) in breakers.iter() {
        let stats = breaker.get_stats().await;
        status.insert(service.clone(), json!({
            "state": format!("{:?}", stats.state),
            "failure_count": stats.failure_count,
            "success_count": stats.success_count,
            "failure_threshold": stats.failure_threshold,
            "recovery_timeout": stats.recovery_timeout
        }));
    }
    
    Ok(Json(json!({
        "circuit_breakers": status,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 限流状态
pub async fn rate_limit_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let config = &state.config.rate_limit;
    
    let response = json!({
        "enabled": config.enabled,
        "requests_per_minute": config.requests_per_minute,
        "burst_size": config.burst_size,
        "window_size": config.window_size,
        "whitelist": config.whitelist,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(response))
}

/// WebSocket统计信息
pub async fn websocket_stats(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state.websocket_manager.get_pool_stats().await;
    
    let response = json!({
        "pool_stats": stats,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(response))
}

/// WebSocket连接信息
pub async fn websocket_connections(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state.websocket_manager.get_pool_stats().await;
    
    let response = json!({
        "total_connections": stats.total_connections,
        "active_connections": stats.active_connections,
        "connections_by_service": stats.connections_by_service,
        "total_messages": stats.total_messages,
        "total_errors": stats.total_errors,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(response))
}