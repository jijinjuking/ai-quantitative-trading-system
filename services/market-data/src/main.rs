mod config;
mod connectors;
mod continuity;
mod handlers;
mod processors;
mod storage;
mod websocket;

use anyhow::Result;
use axum::Router;
use shared_utils::{LoggingInitializer, AppMetrics};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

use crate::{
    config::MarketDataConfig,
    handlers::create_routes,
    processors::DataProcessor,
    storage::StorageManager,
    connectors::ExchangeManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 初始化日志
    LoggingInitializer::init_dev()?;

    // 加载配置
    let config = MarketDataConfig::load()?;
    info!("Market data service configuration loaded");

    // 初始化指标
    let metrics = Arc::new(AppMetrics::new()?);
    info!("Metrics initialized");

    // 初始化存储管理器
    let storage_manager = Arc::new(StorageManager::new(config.clone()).await?);
    info!("Storage manager initialized");

    // 初始化数据处理器
    let data_processor = Arc::new(DataProcessor::new(
        config.clone(),
        storage_manager.clone(),
        metrics.clone(),
    ).await?);
    info!("Data processor initialized");

    // 初始化交易所连接管理器
    let exchange_manager = Arc::new(ExchangeManager::new(
        config.clone(),
        data_processor.clone(),
        metrics.clone(),
    ).await?);
    info!("Exchange manager initialized");

    // 启动交易所连接
    exchange_manager.start_all_connections().await?;
    info!("Exchange connections started");

    // 创建应用状态
    let app_state = AppState {
        config: config.clone(),
        metrics,
        storage_manager,
        data_processor,
        exchange_manager,
    };

    // 创建中间件层
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    // 创建路由
    let app = create_routes()
        .layer(middleware)
        .with_state(app_state);

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 Market Data Service starting on {}", addr);
    info!("📊 Metrics available at http://{}/metrics", addr);
    info!("🏥 Health check available at http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub config: MarketDataConfig,
    pub metrics: Arc<AppMetrics>,
    pub storage_manager: Arc<StorageManager>,
    pub data_processor: Arc<DataProcessor>,
    pub exchange_manager: Arc<ExchangeManager>,
}