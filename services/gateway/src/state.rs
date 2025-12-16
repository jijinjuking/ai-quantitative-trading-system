use anyhow::Result;
use redis::aio::ConnectionManager;
use shared_utils::{AppMetrics, JwtService};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::GatewayConfig;
use crate::services::{CircuitBreaker, ServiceRegistry, RateLimiter};
use crate::websocket::WebSocketManager;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub config: GatewayConfig,
    pub metrics: Arc<AppMetrics>,
    pub jwt_service: Arc<JwtService>,
    pub redis: Arc<RwLock<ConnectionManager>>,
    pub service_registry: Arc<ServiceRegistry>,
    pub rate_limiter: Arc<RateLimiter>,
    pub circuit_breakers: Arc<RwLock<std::collections::HashMap<String, CircuitBreaker>>>,
    pub websocket_manager: Arc<WebSocketManager>,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new(config: GatewayConfig, metrics: Arc<AppMetrics>) -> Result<Self> {
        // 初始化JWT服务
        let jwt_service = Arc::new(JwtService::new(
            &config.auth.jwt_secret,
            config.auth.issuer.clone(),
            config.auth.audience.clone(),
            config.auth.jwt_expiry as i64 / 3600, // 转换为小时
            config.auth.refresh_token_expiry as i64 / 86400, // 转换为天
        ));

        // 初始化Redis连接
        let redis_client = redis::Client::open(config.redis.url.as_str())?;
        let redis_conn = ConnectionManager::new(redis_client).await?;
        let redis = Arc::new(RwLock::new(redis_conn));

        // 初始化服务注册表
        let service_registry = Arc::new(ServiceRegistry::new(config.clone()));

        // 初始化限流器
        let rate_limiter = Arc::new(RateLimiter::new(
            config.rate_limit.clone(),
            redis.clone(),
        ));

        // 初始化熔断器
        let circuit_breakers = Arc::new(RwLock::new(std::collections::HashMap::new()));

        // 初始化WebSocket管理器
        let websocket_manager = Arc::new(WebSocketManager::new());

        Ok(Self {
            config,
            metrics,
            jwt_service,
            redis,
            service_registry,
            rate_limiter,
            circuit_breakers,
            websocket_manager,
        })
    }

    /// 获取或创建熔断器
    pub async fn get_circuit_breaker(&self, service_name: &str) -> CircuitBreaker {
        let mut breakers = self.circuit_breakers.write().await;
        
        if let Some(breaker) = breakers.get(service_name) {
            breaker.clone()
        } else {
            let config = self.config.get_service_endpoint(service_name)
                .map(|endpoint| endpoint.circuit_breaker.clone())
                .unwrap_or_default();
            
            let breaker = CircuitBreaker::new(config);
            breakers.insert(service_name.to_string(), breaker.clone());
            breaker
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> HealthStatus {
        let mut status = HealthStatus {
            status: "healthy".to_string(),
            checks: std::collections::HashMap::new(),
        };

        // 检查Redis连接
        match self.check_redis().await {
            Ok(_) => {
                status.checks.insert("redis".to_string(), ServiceHealth {
                    status: "healthy".to_string(),
                    message: None,
                });
            }
            Err(e) => {
                status.status = "degraded".to_string();
                status.checks.insert("redis".to_string(), ServiceHealth {
                    status: "unhealthy".to_string(),
                    message: Some(e.to_string()),
                });
            }
        }

        // 检查下游服务
        let services = ["user", "trading", "market-data", "strategy", "risk", "notification", "analytics"];
        for service in services {
            match self.service_registry.health_check(service).await {
                Ok(_) => {
                    status.checks.insert(service.to_string(), ServiceHealth {
                        status: "healthy".to_string(),
                        message: None,
                    });
                }
                Err(e) => {
                    if status.status == "healthy" {
                        status.status = "degraded".to_string();
                    }
                    status.checks.insert(service.to_string(), ServiceHealth {
                        status: "unhealthy".to_string(),
                        message: Some(e.to_string()),
                    });
                }
            }
        }

        status
    }

    /// 检查Redis连接
    async fn check_redis(&self) -> Result<()> {
        use redis::AsyncCommands;
        let mut conn = self.redis.write().await;
        // 使用简单的GET命令来测试连接
        let _: Option<String> = conn.get("__health_check__").await?;
        Ok(())
    }
}

/// 健康状态
#[derive(Debug, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub checks: std::collections::HashMap<String, ServiceHealth>,
}

/// 服务健康状态
#[derive(Debug, serde::Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub message: Option<String>,
}