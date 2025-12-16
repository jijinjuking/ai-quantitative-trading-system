mod config;
mod handlers;
mod middleware;
mod routes;
mod services;
mod state;
mod websocket;

use anyhow::Result;
use axum::{extract::connect_info::ConnectInfo, Router};
use shared_utils::{LoggingInitializer, AppMetrics};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

use crate::{
    config::GatewayConfig,
    middleware::{auth::auth_middleware, rate_limit::rate_limit_middleware, request_id::request_id_middleware},
    routes::create_routes,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 初始化日志
    LoggingInitializer::init_dev()?;

    // 加载配置
    let config = GatewayConfig::load()?;
    info!("Gateway configuration loaded");

    // 初始化指标
    let metrics = Arc::new(AppMetrics::new()?);
    info!("Metrics initialized");

    // 创建应用状态
    let state = AppState::new(config.clone(), metrics.clone()).await?;
    info!("Application state initialized");

    // 创建中间件层
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware));

    // 创建路由
    let app = create_routes()
        .layer(middleware)
        .with_state(state);

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 Gateway server starting on {}", addr);
    info!("📊 Metrics available at http://{}/metrics", addr);
    info!("🏥 Health check available at http://{}/health", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await?;

    Ok(())
}