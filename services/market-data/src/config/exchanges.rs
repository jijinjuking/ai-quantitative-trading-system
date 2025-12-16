use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 交易所配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub enabled: bool,
    pub name: String,
    pub websocket_url: String,
    pub rest_api_url: String,
    pub symbols: Vec<String>,
    pub credentials: Option<ExchangeCredentials>,
    pub connection: ConnectionConfig,
    pub rate_limits: RateLimits,
    pub data_types: DataTypes,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            name: String::new(),
            websocket_url: String::new(),
            rest_api_url: String::new(),
            symbols: Vec::new(),
            credentials: None,
            connection: ConnectionConfig::default(),
            rate_limits: RateLimits::default(),
            data_types: DataTypes::default(),
        }
    }
}

/// 交易所认证凭据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCredentials {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: Option<String>,
    pub sandbox: bool,
}

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub connect_timeout: u64,
    pub read_timeout: u64,
    pub write_timeout: u64,
    pub ping_interval: u64,
    pub pong_timeout: u64,
    pub reconnect_interval: u64,
    pub max_reconnect_attempts: u32,
    pub backoff_multiplier: f64,
    pub max_backoff: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 10,
            read_timeout: 30,
            write_timeout: 10,
            ping_interval: 30,
            pong_timeout: 10,
            reconnect_interval: 5,
            max_reconnect_attempts: 10,
            backoff_multiplier: 2.0,
            max_backoff: 300,
        }
    }
}

/// 限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub weight_per_request: u32,
    pub max_weight_per_minute: u32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            requests_per_minute: 1200,
            weight_per_request: 1,
            max_weight_per_minute: 6000,
        }
    }
}

/// 数据类型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTypes {
    pub ticker: bool,
    pub kline: bool,
    pub depth: bool,
    pub trade: bool,
    pub kline_intervals: Vec<String>,
    pub depth_levels: u32,
}

impl Default for DataTypes {
    fn default() -> Self {
        Self {
            ticker: true,
            kline: true,
            depth: true,
            trade: true,
            kline_intervals: vec![
                "1m".to_string(),
                "5m".to_string(),
                "15m".to_string(),
                "1h".to_string(),
                "4h".to_string(),
                "1d".to_string(),
            ],
            depth_levels: 20,
        }
    }
}

impl ExchangeConfig {
    /// 创建币安配置
    pub fn binance() -> Self {
        Self {
            enabled: true,
            name: "binance".to_string(),
            websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
            rest_api_url: "https://api.binance.com".to_string(),
            symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "BNBUSDT".to_string(),
            ],
            credentials: None,
            connection: ConnectionConfig::default(),
            rate_limits: RateLimits {
                requests_per_second: 10,
                requests_per_minute: 1200,
                weight_per_request: 1,
                max_weight_per_minute: 6000,
            },
            data_types: DataTypes::default(),
        }
    }

    /// 创建OKX配置
    pub fn okx() -> Self {
        Self {
            enabled: true,
            name: "okx".to_string(),
            websocket_url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            rest_api_url: "https://www.okx.com".to_string(),
            symbols: vec![
                "BTC-USDT".to_string(),
                "ETH-USDT".to_string(),
                "OKB-USDT".to_string(),
            ],
            credentials: None,
            connection: ConnectionConfig::default(),
            rate_limits: RateLimits {
                requests_per_second: 20,
                requests_per_minute: 600,
                weight_per_request: 1,
                max_weight_per_minute: 2400,
            },
            data_types: DataTypes::default(),
        }
    }

    /// 创建火币配置
    pub fn huobi() -> Self {
        Self {
            enabled: true,
            name: "huobi".to_string(),
            websocket_url: "wss://api.huobi.pro/ws".to_string(),
            rest_api_url: "https://api.huobi.pro".to_string(),
            symbols: vec![
                "btcusdt".to_string(),
                "ethusdt".to_string(),
                "htusdt".to_string(),
            ],
            credentials: None,
            connection: ConnectionConfig::default(),
            rate_limits: RateLimits {
                requests_per_second: 10,
                requests_per_minute: 100,
                weight_per_request: 1,
                max_weight_per_minute: 2400,
            },
            data_types: DataTypes::default(),
        }
    }

    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Exchange name cannot be empty"));
        }

        if self.websocket_url.is_empty() {
            return Err(anyhow::anyhow!("WebSocket URL cannot be empty"));
        }

        if self.rest_api_url.is_empty() {
            return Err(anyhow::anyhow!("REST API URL cannot be empty"));
        }

        if self.symbols.is_empty() {
            return Err(anyhow::anyhow!("At least one symbol must be configured"));
        }

        // 验证WebSocket URL格式
        if !self.websocket_url.starts_with("ws://") && !self.websocket_url.starts_with("wss://") {
            return Err(anyhow::anyhow!("Invalid WebSocket URL format"));
        }

        // 验证REST API URL格式
        if !self.rest_api_url.starts_with("http://") && !self.rest_api_url.starts_with("https://") {
            return Err(anyhow::anyhow!("Invalid REST API URL format"));
        }

        Ok(())
    }

    /// 获取连接超时时间
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connection.connect_timeout)
    }

    /// 获取读取超时时间
    pub fn read_timeout(&self) -> Duration {
        Duration::from_secs(self.connection.read_timeout)
    }

    /// 获取写入超时时间
    pub fn write_timeout(&self) -> Duration {
        Duration::from_secs(self.connection.write_timeout)
    }

    /// 获取心跳间隔
    pub fn ping_interval(&self) -> Duration {
        Duration::from_secs(self.connection.ping_interval)
    }

    /// 获取重连间隔
    pub fn reconnect_interval(&self) -> Duration {
        Duration::from_secs(self.connection.reconnect_interval)
    }

    /// 计算退避延迟
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base_delay = self.connection.reconnect_interval as f64;
        let multiplier = self.connection.backoff_multiplier;
        let max_delay = self.connection.max_backoff as f64;

        let delay = base_delay * multiplier.powi(attempt as i32);
        let capped_delay = delay.min(max_delay);

        Duration::from_secs(capped_delay as u64)
    }

    /// 检查是否启用了指定数据类型
    pub fn is_data_type_enabled(&self, data_type: &str) -> bool {
        match data_type {
            "ticker" => self.data_types.ticker,
            "kline" => self.data_types.kline,
            "depth" => self.data_types.depth,
            "trade" => self.data_types.trade,
            _ => false,
        }
    }
}

/// 预定义的交易所配置
pub fn default_exchanges() -> HashMap<String, ExchangeConfig> {
    let mut exchanges = HashMap::new();

    exchanges.insert("binance".to_string(), ExchangeConfig::binance());
    exchanges.insert("okx".to_string(), ExchangeConfig::okx());
    exchanges.insert("huobi".to_string(), ExchangeConfig::huobi());

    exchanges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_config_validation() {
        let mut config = ExchangeConfig::default();

        // 空名称应该失败
        assert!(config.validate().is_err());

        // 设置有效配置
        config.name = "test".to_string();
        config.websocket_url = "wss://example.com/ws".to_string();
        config.rest_api_url = "https://api.example.com".to_string();
        config.symbols = vec!["BTCUSDT".to_string()];

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_binance_config() {
        let config = ExchangeConfig::binance();
        assert_eq!(config.name, "binance");
        assert!(config.validate().is_ok());
        assert!(config.symbols.contains(&"BTCUSDT".to_string()));
    }

    #[test]
    fn test_backoff_calculation() {
        let config = ExchangeConfig::default();

        let delay1 = config.calculate_backoff(0);
        let delay2 = config.calculate_backoff(1);
        let delay3 = config.calculate_backoff(2);

        assert!(delay2 > delay1);
        assert!(delay3 > delay2);

        // 测试最大延迟限制
        let max_delay = config.calculate_backoff(100);
        assert_eq!(max_delay.as_secs(), config.connection.max_backoff);
    }

    #[test]
    fn test_data_type_enabled() {
        let config = ExchangeConfig::default();

        assert!(config.is_data_type_enabled("ticker"));
        assert!(config.is_data_type_enabled("kline"));
        assert!(config.is_data_type_enabled("depth"));
        assert!(config.is_data_type_enabled("trade"));
        assert!(!config.is_data_type_enabled("invalid"));
    }
}
