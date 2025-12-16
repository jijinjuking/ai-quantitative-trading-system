pub mod auth;
pub mod health;
pub mod metrics;
pub mod proxy;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

/// 创建所有路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 健康检查和指标
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::metrics_handler))
        // 认证路由
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/refresh", post(auth::refresh_token))
        // 服务代理路由
        .route(
            "/api/v1/:service/*path",
            get(proxy::proxy_get)
                .post(proxy::proxy_post)
                .put(proxy::proxy_put)
                .delete(proxy::proxy_delete)
                .patch(proxy::proxy_patch),
        )
        // WebSocket代理
        .route("/ws/:service/*path", get(proxy::proxy_websocket))
        // 管理接口
        .route("/admin/services", get(health::list_services))
        .route(
            "/admin/circuit-breakers",
            get(health::circuit_breaker_status),
        )
        .route("/admin/rate-limits", get(health::rate_limit_status))
        .route("/admin/websocket/stats", get(health::websocket_stats))
        .route(
            "/admin/websocket/connections",
            get(health::websocket_connections),
        )
}
