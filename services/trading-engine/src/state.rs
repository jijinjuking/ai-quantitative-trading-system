use anyhow::Result;
use shared_utils::AppMetrics;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    config::TradingEngineConfig,
    services::{AccountService, ExecutionService, OrderService, PositionService, RiskService},
    storage::{AccountStore, OrderStore, PositionStore, TradeStore},
};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub config: TradingEngineConfig,
    pub metrics: Arc<AppMetrics>,
    pub db_pool: Arc<PgPool>,
    
    // 存储层
    pub order_store: Arc<OrderStore>,
    pub position_store: Arc<PositionStore>,
    pub account_store: Arc<AccountStore>,
    pub trade_store: Arc<TradeStore>,
    
    // 服务层
    pub order_service: Arc<OrderService>,
    pub position_service: Arc<PositionService>,
    pub account_service: Arc<AccountService>,
    pub execution_service: Arc<ExecutionService>,
    pub risk_service: Arc<RiskService>,
}

impl AppState {
    pub async fn new(config: TradingEngineConfig, metrics: Arc<AppMetrics>) -> Result<Self> {
        // 创建数据库连接池
        let db_pool = Arc::new(
            PgPool::connect(&config.database.url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?,
        );

        // 创建存储层
        let order_store = Arc::new(OrderStore::new(db_pool.clone()));
        let position_store = Arc::new(PositionStore::new(db_pool.clone()));
        let account_store = Arc::new(AccountStore::new(db_pool.clone()));
        let trade_store = Arc::new(TradeStore::new(db_pool.clone()));

        // 创建服务层
        let execution_service = Arc::new(ExecutionService::new(config.clone()).await?);
        let risk_service = Arc::new(RiskService::new(config.clone()));
        
        let order_service = Arc::new(OrderService::new(
            order_store.clone(),
            execution_service.clone(),
            risk_service.clone(),
        ));
        
        let position_service = Arc::new(PositionService::new(
            position_store.clone(),
            execution_service.clone(),
            risk_service.clone(),
        ));
        
        let account_service = Arc::new(AccountService::new(
            account_store.clone(),
            position_service.clone(),
        ));

        Ok(Self {
            config,
            metrics,
            db_pool,
            order_store,
            position_store,
            account_store,
            trade_store,
            order_service,
            position_service,
            account_service,
            execution_service,
            risk_service,
        })
    }

    /// 检查数据库健康状态
    pub async fn check_database_health(&self) -> String {
        match sqlx::query("SELECT 1").fetch_one(&*self.db_pool).await {
            Ok(_) => "healthy".to_string(),
            Err(e) => format!("unhealthy: {}", e),
        }
    }

    /// 检查Redis健康状态
    pub async fn check_redis_health(&self) -> String {
        // TODO: 实现Redis健康检查
        "healthy".to_string()
    }

    /// 检查Kafka健康状态
    pub async fn check_kafka_health(&self) -> String {
        // TODO: 实现Kafka健康检查
        "healthy".to_string()
    }
}