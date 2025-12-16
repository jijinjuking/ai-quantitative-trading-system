pub mod execution;
pub mod risk;
pub mod trading;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub use execution::ExecutionConfig;
pub use risk::RiskConfig;
pub use trading::TradingConfig;

/// 交易引擎主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingEngineConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub execution: ExecutionConfig,
    pub websocket: WebSocketConfig,
    pub monitoring: MonitoringConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: usize,
    pub request_timeout: Duration,
    pub shutdown_timeout: Duration,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

/// Redis配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connect_timeout: Duration,
    pub command_timeout: Duration,
    pub connection_timeout: Duration,
}

/// WebSocket配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub max_connections: usize,
    pub heartbeat_interval: Duration,
    pub message_timeout: Duration,
    pub buffer_size: usize,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics_path: String,
    pub health_path: String,
    pub prometheus_registry: bool,
}

impl TradingEngineConfig {
    /// 加载配置
    pub fn load() -> Result<Self> {
        let mut settings = config::Config::builder()
            .add_source(config::File::with_name("config/development").required(false))
            .add_source(config::Environment::with_prefix("TRADING_ENGINE").separator("__"))
            .build()?;

        // 设置默认值
        settings.set_default("server.host", "0.0.0.0")?;
        settings.set_default("server.port", 8082)?;
        settings.set_default("server.max_connections", 1000)?;
        settings.set_default("server.request_timeout", "30s")?;
        settings.set_default("server.shutdown_timeout", "10s")?;

        settings.set_default("database.max_connections", 20)?;
        settings.set_default("database.min_connections", 5)?;
        settings.set_default("database.connect_timeout", "10s")?;
        settings.set_default("database.idle_timeout", "300s")?;
        settings.set_default("database.max_lifetime", "1800s")?;

        settings.set_default("redis.pool_size", 10)?;
        settings.set_default("redis.connect_timeout", "5s")?;
        settings.set_default("redis.command_timeout", "3s")?;
        settings.set_default("redis.connection_timeout", "300s")?;

        settings.set_default("websocket.enabled", true)?;
        settings.set_default("websocket.max_connections", 1000)?;
        settings.set_default("websocket.heartbeat_interval", "30s")?;
        settings.set_default("websocket.message_timeout", "10s")?;
        settings.set_default("websocket.buffer_size", 1024)?;

        settings.set_default("monitoring.enabled", true)?;
        settings.set_default("monitoring.metrics_path", "/metrics")?;
        settings.set_default("monitoring.health_path", "/health")?;
        settings.set_default("monitoring.prometheus_registry", true)?;

        // 交易配置默认值
        settings.set_default("trading.enabled", true)?;
        settings.set_default("trading.max_orders_per_user", 100)?;
        settings.set_default("trading.max_order_size", "1000000")?;
        settings.set_default("trading.min_order_size", "0.001")?;
        settings.set_default("trading.order_timeout", "86400s")?; // 24小时

        // 风控配置默认值
        settings.set_default("risk.enabled", true)?;
        settings.set_default("risk.max_position_size", "10000000")?;
        settings.set_default("risk.max_daily_loss", "100000")?;
        settings.set_default("risk.max_leverage", "10")?;
        settings.set_default("risk.margin_call_threshold", "0.8")?;
        settings.set_default("risk.liquidation_threshold", "0.9")?;

        // 执行配置默认值
        settings.set_default("execution.enabled", true)?;
        settings.set_default("execution.max_slippage", "0.01")?;
        settings.set_default("execution.execution_timeout", "5s")?;
        settings.set_default("execution.retry_attempts", 3)?;
        settings.set_default("execution.retry_delay", "1s")?;

        let config: TradingEngineConfig = settings.try_deserialize()?;
        Ok(config)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // 验证服务器配置
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        if self.server.max_connections == 0 {
            return Err(anyhow::anyhow!("Max connections cannot be 0"));
        }

        // 验证数据库配置
        if self.database.url.is_empty() {
            return Err(anyhow::anyhow!("Database URL is required"));
        }

        if self.database.max_connections == 0 {
            return Err(anyhow::anyhow!("Database max connections cannot be 0"));
        }

        // 验证Redis配置
        if self.redis.url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL is required"));
        }

        // 验证交易配置
        self.trading.validate()?;
        self.risk.validate()?;
        self.execution.validate()?;

        Ok(())
    }
}

impl Default for TradingEngineConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8082,
                workers: None,
                max_connections: 1000,
                request_timeout: Duration::from_secs(30),
                shutdown_timeout: Duration::from_secs(10),
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/trading_engine".to_string(),
                max_connections: 20,
                min_connections: 5,
                connect_timeout: Duration::from_secs(10),
                idle_timeout: Duration::from_secs(300),
                max_lifetime: Duration::from_secs(1800),
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                connect_timeout: Duration::from_secs(5),
                command_timeout: Duration::from_secs(3),
                connection_timeout: Duration::from_secs(300),
            },
            trading: TradingConfig::default(),
            risk: RiskConfig::default(),
            execution: ExecutionConfig::default(),
            websocket: WebSocketConfig {
                enabled: true,
                max_connections: 1000,
                heartbeat_interval: Duration::from_secs(30),
                message_timeout: Duration::from_secs(10),
                buffer_size: 1024,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_path: "/metrics".to_string(),
                health_path: "/health".to_string(),
                prometheus_registry: true,
            },
        }
    }
}
