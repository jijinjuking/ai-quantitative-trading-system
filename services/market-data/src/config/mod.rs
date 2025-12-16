pub mod exchanges;
pub mod server;
pub mod storage;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use exchanges::{ExchangeConfig, ExchangeCredentials};
pub use server::ServerConfig;
pub use storage::{ClickHouseConfig, RedisConfig, StorageConfig};

/// 市场数据服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    pub server: ServerConfig,
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub storage: StorageConfig,
    pub data_processing: DataProcessingConfig,
    pub websocket: WebSocketConfig,
    pub monitoring: MonitoringConfig,
}

impl MarketDataConfig {
    /// 从环境变量和配置文件加载配置
    pub fn load() -> Result<Self> {
        let mut settings = config::Config::builder()
            .add_source(config::File::with_name("config/market-data").required(false))
            .add_source(config::Environment::with_prefix("MARKET_DATA").separator("__"))
            .build()?;

        // 设置默认值
        settings.set_default("server.host", "0.0.0.0")?;
        settings.set_default("server.port", 8081)?;
        settings.set_default("data_processing.batch_size", 1000)?;
        settings.set_default("data_processing.flush_interval", 5)?;
        settings.set_default("websocket.max_connections", 10000)?;
        settings.set_default("monitoring.metrics_enabled", true)?;

        let config: MarketDataConfig = settings.try_deserialize()?;
        config.validate()?;

        Ok(config)
    }

    /// 验证配置
    fn validate(&self) -> Result<()> {
        if self.exchanges.is_empty() {
            return Err(anyhow::anyhow!("At least one exchange must be configured"));
        }

        for (name, exchange_config) in &self.exchanges {
            if exchange_config.symbols.is_empty() {
                return Err(anyhow::anyhow!(
                    "Exchange {} must have at least one symbol",
                    name
                ));
            }
        }

        Ok(())
    }

    /// 获取所有启用的交易所
    pub fn enabled_exchanges(&self) -> Vec<(&String, &ExchangeConfig)> {
        self.exchanges
            .iter()
            .filter(|(_, config)| config.enabled)
            .collect()
    }

    /// 获取所有交易对
    pub fn all_symbols(&self) -> Vec<String> {
        let mut symbols = std::collections::HashSet::new();

        for exchange_config in self.exchanges.values() {
            if exchange_config.enabled {
                for symbol in &exchange_config.symbols {
                    symbols.insert(symbol.clone());
                }
            }
        }

        symbols.into_iter().collect()
    }
}

/// 数据处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingConfig {
    pub batch_size: usize,
    pub flush_interval: u64,
    pub max_queue_size: usize,
    pub enable_data_validation: bool,
    pub enable_duplicate_detection: bool,
    pub data_retention_days: u32,
}

impl Default for DataProcessingConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            flush_interval: 5,
            max_queue_size: 100000,
            enable_data_validation: true,
            enable_duplicate_detection: true,
            data_retention_days: 365,
        }
    }
}

/// WebSocket配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub max_connections: usize,
    pub heartbeat_interval: u64,
    pub message_buffer_size: usize,
    pub compression_enabled: bool,
    pub rate_limit: WebSocketRateLimit,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 10000,
            heartbeat_interval: 30,
            message_buffer_size: 1000,
            compression_enabled: true,
            rate_limit: WebSocketRateLimit::default(),
        }
    }
}

/// WebSocket限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketRateLimit {
    pub enabled: bool,
    pub messages_per_second: u32,
    pub burst_size: u32,
}

impl Default for WebSocketRateLimit {
    fn default() -> Self {
        Self {
            enabled: true,
            messages_per_second: 100,
            burst_size: 200,
        }
    }
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub health_check_interval: u64,
    pub alert_thresholds: AlertThresholds,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            health_check_interval: 30,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// 告警阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_data_delay_ms: u64,
    pub min_connection_uptime_percent: f64,
    pub max_error_rate_percent: f64,
    pub max_memory_usage_mb: u64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_data_delay_ms: 5000,
            min_connection_uptime_percent: 95.0,
            max_error_rate_percent: 5.0,
            max_memory_usage_mb: 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = MarketDataConfig {
            server: ServerConfig::default(),
            exchanges: HashMap::new(),
            storage: StorageConfig::default(),
            data_processing: DataProcessingConfig::default(),
            websocket: WebSocketConfig::default(),
            monitoring: MonitoringConfig::default(),
        };

        // 空交易所配置应该失败
        assert!(config.validate().is_err());

        // 添加交易所配置
        config.exchanges.insert(
            "binance".to_string(),
            ExchangeConfig {
                enabled: true,
                symbols: vec!["BTCUSDT".to_string()],
                ..Default::default()
            },
        );

        // 现在应该通过验证
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_enabled_exchanges() {
        let mut config = MarketDataConfig {
            server: ServerConfig::default(),
            exchanges: HashMap::new(),
            storage: StorageConfig::default(),
            data_processing: DataProcessingConfig::default(),
            websocket: WebSocketConfig::default(),
            monitoring: MonitoringConfig::default(),
        };

        // 添加启用的交易所
        config.exchanges.insert(
            "binance".to_string(),
            ExchangeConfig {
                enabled: true,
                symbols: vec!["BTCUSDT".to_string()],
                ..Default::default()
            },
        );

        // 添加禁用的交易所
        config.exchanges.insert(
            "okx".to_string(),
            ExchangeConfig {
                enabled: false,
                symbols: vec!["ETHUSDT".to_string()],
                ..Default::default()
            },
        );

        let enabled = config.enabled_exchanges();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].0, "binance");
    }
}
