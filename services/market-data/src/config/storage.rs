use serde::{Deserialize, Serialize};

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub clickhouse: Option<ClickHouseConfig>,
    pub redis: Option<RedisConfig>,
    pub kafka: Option<KafkaConfig>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            clickhouse: Some(ClickHouseConfig::default()),
            redis: Some(RedisConfig::default()),
            kafka: Some(KafkaConfig::default()),
        }
    }
}

/// ClickHouse配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub connection_timeout: u64,
    pub query_timeout: u64,
    pub max_connections: u32,
    pub batch_size: usize,
    pub compression: bool,
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8123".to_string(),
            database: "market_data".to_string(),
            username: "default".to_string(),
            password: String::new(),
            connection_timeout: 30,
            query_timeout: 60,
            max_connections: 10,
            batch_size: 10000,
            compression: true,
        }
    }
}

impl ClickHouseConfig {
    /// 构建连接URL
    pub fn connection_url(&self) -> String {
        if self.password.is_empty() {
            format!(
                "{}?database={}&user={}",
                self.url, self.database, self.username
            )
        } else {
            format!(
                "{}?database={}&user={}&password={}",
                self.url, self.database, self.username, self.password
            )
        }
    }

    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("ClickHouse URL cannot be empty"));
        }

        if self.database.is_empty() {
            return Err(anyhow::anyhow!("ClickHouse database cannot be empty"));
        }

        if self.username.is_empty() {
            return Err(anyhow::anyhow!("ClickHouse username cannot be empty"));
        }

        if self.max_connections == 0 {
            return Err(anyhow::anyhow!(
                "ClickHouse max connections must be greater than 0"
            ));
        }

        if self.batch_size == 0 {
            return Err(anyhow::anyhow!(
                "ClickHouse batch size must be greater than 0"
            ));
        }

        Ok(())
    }
}

/// Redis配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    pub database: u8,
    pub connection_timeout: u64,
    pub max_connections: u32,
    pub ttl_seconds: u64,
    pub key_prefix: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            password: None,
            database: 0,
            connection_timeout: 30,
            max_connections: 20,
            ttl_seconds: 3600, // 1小时
            key_prefix: "market_data:".to_string(),
        }
    }
}

impl RedisConfig {
    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL cannot be empty"));
        }

        if !self.url.starts_with("redis://") && !self.url.starts_with("rediss://") {
            return Err(anyhow::anyhow!("Invalid Redis URL format"));
        }

        if self.max_connections == 0 {
            return Err(anyhow::anyhow!(
                "Redis max connections must be greater than 0"
            ));
        }

        Ok(())
    }

    /// 生成缓存键
    pub fn cache_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    /// 生成价格缓存键
    pub fn price_key(&self, exchange: &str, symbol: &str) -> String {
        self.cache_key(&format!("price:{}:{}", exchange, symbol))
    }

    /// 生成K线缓存键
    pub fn kline_key(&self, exchange: &str, symbol: &str, interval: &str) -> String {
        self.cache_key(&format!("kline:{}:{}:{}", exchange, symbol, interval))
    }

    /// 生成深度缓存键
    pub fn depth_key(&self, exchange: &str, symbol: &str) -> String {
        self.cache_key(&format!("depth:{}:{}", exchange, symbol))
    }
}

/// Kafka配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub topic_prefix: String,
    pub batch_size: usize,
    pub linger_ms: u64,
    pub compression_type: String,
    pub acks: String,
    pub retries: u32,
    pub security_protocol: Option<String>,
    pub sasl_mechanism: Option<String>,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: vec!["localhost:9092".to_string()],
            topic_prefix: "market_data".to_string(),
            batch_size: 16384,
            linger_ms: 5,
            compression_type: "snappy".to_string(),
            acks: "1".to_string(),
            retries: 3,
            security_protocol: None,
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
        }
    }
}

impl KafkaConfig {
    /// 获取broker列表字符串
    pub fn broker_list(&self) -> String {
        self.brokers.join(",")
    }

    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.brokers.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one Kafka broker must be configured"
            ));
        }

        for broker in &self.brokers {
            if broker.is_empty() {
                return Err(anyhow::anyhow!("Kafka broker address cannot be empty"));
            }
        }

        if self.topic_prefix.is_empty() {
            return Err(anyhow::anyhow!("Kafka topic prefix cannot be empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clickhouse_config() {
        let config = ClickHouseConfig::default();
        assert!(config.validate().is_ok());

        let url = config.connection_url();
        assert!(url.contains("database=market_data"));
        assert!(url.contains("user=default"));
    }

    #[test]
    fn test_redis_config() {
        let config = RedisConfig::default();
        assert!(config.validate().is_ok());

        let price_key = config.price_key("binance", "BTCUSDT");
        assert_eq!(price_key, "market_data:price:binance:BTCUSDT");

        let kline_key = config.kline_key("binance", "BTCUSDT", "1m");
        assert_eq!(kline_key, "market_data:kline:binance:BTCUSDT:1m");
    }

    #[test]
    fn test_kafka_config() {
        let config = KafkaConfig::default();
        assert!(config.validate().is_ok());

        let broker_list = config.broker_list();
        assert_eq!(broker_list, "localhost:9092");
    }

    #[test]
    fn test_storage_config_validation() {
        let mut clickhouse_config = ClickHouseConfig::default();
        clickhouse_config.url = String::new();
        assert!(clickhouse_config.validate().is_err());

        let mut redis_config = RedisConfig::default();
        redis_config.url = "invalid-url".to_string();
        assert!(redis_config.validate().is_err());

        let mut kafka_config = KafkaConfig::default();
        kafka_config.brokers.clear();
        assert!(kafka_config.validate().is_err());
    }
}
