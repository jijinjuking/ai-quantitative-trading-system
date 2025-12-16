use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub jwt: JwtConfig,
    pub logging: LoggingConfig,
    pub exchanges: ExchangeConfigs,
    pub risk: RiskConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
    pub timeout: Option<u64>,
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            max_connections: Some(1000),
            timeout: Some(30),
            cors_origins: vec!["*".to_string()],
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub clickhouse_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            postgres_url: "postgresql://admin:password@localhost:5432/trading_platform".to_string(),
            clickhouse_url: "clickhouse://localhost:9000/market_data".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        }
    }
}

/// Redis配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub response_timeout: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            response_timeout: 5,
        }
    }
}

/// Kafka配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub group_id: String,
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub session_timeout: u32,
    pub heartbeat_interval: u32,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: vec!["localhost:9092".to_string()],
            group_id: "trading-platform".to_string(),
            auto_offset_reset: "earliest".to_string(),
            enable_auto_commit: true,
            session_timeout: 30000,
            heartbeat_interval: 3000,
        }
    }
}

/// JWT配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-secret-key".to_string(),
            issuer: "trading-platform".to_string(),
            audience: "trading-platform-users".to_string(),
            access_token_expiry_hours: 1,
            refresh_token_expiry_days: 7,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_enabled: bool,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_files: Option<u32>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file_enabled: false,
            file_path: None,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_files: Some(10),
        }
    }
}

/// 交易所配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfigs {
    pub binance: ExchangeConfig,
    pub okx: ExchangeConfig,
    pub huobi: ExchangeConfig,
}

impl Default for ExchangeConfigs {
    fn default() -> Self {
        Self {
            binance: ExchangeConfig::default(),
            okx: ExchangeConfig::default(),
            huobi: ExchangeConfig::default(),
        }
    }
}

/// 单个交易所配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub enabled: bool,
    pub api_url: String,
    pub ws_url: String,
    pub testnet: bool,
    pub rate_limit: RateLimitConfig,
    pub timeout: u64,
    pub retry_attempts: u32,
    pub retry_delay: u64,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_url: "".to_string(),
            ws_url: "".to_string(),
            testnet: false,
            rate_limit: RateLimitConfig::default(),
            timeout: 30,
            retry_attempts: 3,
            retry_delay: 1000,
        }
    }
}

/// 限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            requests_per_minute: 1200,
            burst_size: 20,
        }
    }
}

/// 风险配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub enabled: bool,
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub max_drawdown: f64,
    pub max_leverage: f64,
    pub check_interval: u64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_position_size: 10000.0,
            max_daily_loss: 1000.0,
            max_drawdown: 0.2,
            max_leverage: 10.0,
            check_interval: 1000,
        }
    }
}

/// 配置加载器
pub struct ConfigLoader;

impl ConfigLoader {
    /// 加载配置
    pub fn load() -> Result<AppConfig> {
        let mut config = Config::builder();

        // 加载默认配置
        config = config.add_source(Config::try_from(&AppConfig::default())?);

        // 加载配置文件
        if let Ok(config_path) = env::var("CONFIG_PATH") {
            config = config.add_source(File::with_name(&config_path).required(false));
        } else {
            config = config
                .add_source(File::with_name("config/default").required(false))
                .add_source(File::with_name("config/local").required(false));
        }

        // 加载环境变量
        config = config.add_source(
            Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        );

        let config = config.build()?;
        let app_config: AppConfig = config.try_deserialize()?;

        Ok(app_config)
    }

    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> Result<AppConfig> {
        let config = Config::builder()
            .add_source(Config::try_from(&AppConfig::default())?)
            .add_source(File::with_name(path))
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;
        Ok(app_config)
    }

    /// 从环境变量加载配置
    pub fn load_from_env() -> Result<AppConfig> {
        let config = Config::builder()
            .add_source(Config::try_from(&AppConfig::default())?)
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;
        Ok(app_config)
    }

    /// 验证配置
    pub fn validate(config: &AppConfig) -> Result<()> {
        // 验证服务器配置
        if config.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        // 验证数据库配置
        if config.database.postgres_url.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL URL cannot be empty"));
        }

        if config.database.clickhouse_url.is_empty() {
            return Err(anyhow::anyhow!("ClickHouse URL cannot be empty"));
        }

        // 验证Redis配置
        if config.redis.url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL cannot be empty"));
        }

        // 验证JWT配置
        if config.jwt.secret.is_empty() || config.jwt.secret == "your-secret-key" {
            return Err(anyhow::anyhow!("JWT secret must be set and not default"));
        }

        // 验证Kafka配置
        if config.kafka.brokers.is_empty() {
            return Err(anyhow::anyhow!("Kafka brokers cannot be empty"));
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            kafka: KafkaConfig::default(),
            jwt: JwtConfig::default(),
            logging: LoggingConfig::default(),
            exchanges: ExchangeConfigs::default(),
            risk: RiskConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(ConfigLoader::validate(&config).is_err()); // Should fail due to default JWT secret
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.jwt.secret = "test-secret-key".to_string();

        assert!(ConfigLoader::validate(&config).is_ok());

        config.server.port = 0;
        assert!(ConfigLoader::validate(&config).is_err());
    }
}
