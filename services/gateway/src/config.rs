use anyhow::Result;
use serde::{Deserialize, Serialize};
use shared_utils::ConfigLoader;
use std::collections::HashMap;

/// 网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub services: ServicesConfig,
    pub redis: RedisConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
    pub timeout: Option<u64>,
    pub request_timeout: u64,
    pub keep_alive: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            max_connections: Some(1000),
            timeout: Some(30),
            request_timeout: 30,
            keep_alive: 60,
        }
    }
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry: u64,
    pub refresh_token_expiry: u64,
    pub issuer: String,
    pub audience: String,
    pub public_paths: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key".to_string(),
            jwt_expiry: 3600,             // 1 hour
            refresh_token_expiry: 604800, // 7 days
            issuer: "trading-platform-gateway".to_string(),
            audience: "trading-platform-users".to_string(),
            public_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/api/v1/auth/login".to_string(),
                "/api/v1/auth/register".to_string(),
                "/api/v1/auth/refresh".to_string(),
                "/api/v1/market/public".to_string(),
            ],
        }
    }
}

/// 限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size: u64,
    pub cleanup_interval: u64,
    pub whitelist: Vec<String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 100,
            burst_size: 20,
            window_size: 60,       // seconds
            cleanup_interval: 300, // 5 minutes
            whitelist: vec!["127.0.0.1".to_string()],
        }
    }
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub user_service: ServiceEndpoint,
    pub trading_service: ServiceEndpoint,
    pub market_data_service: ServiceEndpoint,
    pub strategy_service: ServiceEndpoint,
    pub risk_service: ServiceEndpoint,
    pub notification_service: ServiceEndpoint,
    pub analytics_service: ServiceEndpoint,
}

impl Default for ServicesConfig {
    fn default() -> Self {
        Self {
            user_service: ServiceEndpoint {
                url: "http://localhost:8081".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
            trading_service: ServiceEndpoint {
                url: "http://localhost:8082".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
            market_data_service: ServiceEndpoint {
                url: "http://localhost:8083".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
            strategy_service: ServiceEndpoint {
                url: "http://localhost:8084".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
            risk_service: ServiceEndpoint {
                url: "http://localhost:8085".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
            notification_service: ServiceEndpoint {
                url: "http://localhost:8086".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
            analytics_service: ServiceEndpoint {
                url: "http://localhost:8087".to_string(),
                timeout: 30,
                retries: 3,
                circuit_breaker: CircuitBreakerConfig::default(),
            },
        }
    }
}

/// 服务端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub url: String,
    pub timeout: u64,
    pub retries: u32,
    pub circuit_breaker: CircuitBreakerConfig,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: u64,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: 60,
            half_open_max_calls: 3,
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
    pub key_prefix: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            response_timeout: 5,
            key_prefix: "gateway:".to_string(),
        }
    }
}

/// CORS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
                "X-Request-ID".to_string(),
            ],
            max_age: 3600,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub access_log: bool,
    pub error_log: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            access_log: true,
            error_log: true,
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            services: ServicesConfig::default(),
            redis: RedisConfig::default(),
            cors: CorsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl GatewayConfig {
    /// 加载配置
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // 从环境变量加载配置
        if let Ok(host) = std::env::var("GATEWAY_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("GATEWAY_PORT") {
            config.server.port = port.parse()?;
        }
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            config.redis.url = redis_url;
        }

        // 服务端点配置
        if let Ok(user_service_url) = std::env::var("USER_SERVICE_URL") {
            config.services.user_service.url = user_service_url;
        }
        if let Ok(trading_service_url) = std::env::var("TRADING_SERVICE_URL") {
            config.services.trading_service.url = trading_service_url;
        }
        if let Ok(market_data_service_url) = std::env::var("MARKET_DATA_SERVICE_URL") {
            config.services.market_data_service.url = market_data_service_url;
        }
        if let Ok(strategy_service_url) = std::env::var("STRATEGY_SERVICE_URL") {
            config.services.strategy_service.url = strategy_service_url;
        }
        if let Ok(risk_service_url) = std::env::var("RISK_SERVICE_URL") {
            config.services.risk_service.url = risk_service_url;
        }
        if let Ok(notification_service_url) = std::env::var("NOTIFICATION_SERVICE_URL") {
            config.services.notification_service.url = notification_service_url;
        }
        if let Ok(analytics_service_url) = std::env::var("ANALYTICS_SERVICE_URL") {
            config.services.analytics_service.url = analytics_service_url;
        }

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        if self.auth.jwt_secret.is_empty() || self.auth.jwt_secret == "your-secret-key" {
            return Err(anyhow::anyhow!("JWT secret must be set and not default"));
        }

        if self.redis.url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL cannot be empty"));
        }

        // 验证服务端点
        let services = [
            &self.services.user_service,
            &self.services.trading_service,
            &self.services.market_data_service,
            &self.services.strategy_service,
            &self.services.risk_service,
            &self.services.notification_service,
            &self.services.analytics_service,
        ];

        for service in services {
            if service.url.is_empty() {
                return Err(anyhow::anyhow!("Service URL cannot be empty"));
            }
            if !service.url.starts_with("http://") && !service.url.starts_with("https://") {
                return Err(anyhow::anyhow!(
                    "Service URL must start with http:// or https://"
                ));
            }
        }

        Ok(())
    }

    /// 获取服务端点
    pub fn get_service_endpoint(&self, service_name: &str) -> Option<&ServiceEndpoint> {
        match service_name {
            "user" => Some(&self.services.user_service),
            "trading" => Some(&self.services.trading_service),
            "market-data" => Some(&self.services.market_data_service),
            "strategy" => Some(&self.services.strategy_service),
            "risk" => Some(&self.services.risk_service),
            "notification" => Some(&self.services.notification_service),
            "analytics" => Some(&self.services.analytics_service),
            _ => None,
        }
    }

    /// 检查路径是否为公开路径
    pub fn is_public_path(&self, path: &str) -> bool {
        self.auth.public_paths.iter().any(|public_path| {
            if public_path.ends_with('*') {
                path.starts_with(&public_path[..public_path.len() - 1])
            } else {
                path == public_path
            }
        })
    }

    /// 检查IP是否在白名单中
    pub fn is_whitelisted_ip(&self, ip: &str) -> bool {
        self.rate_limit.whitelist.contains(&ip.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(config.rate_limit.enabled);
        assert!(config.cors.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = GatewayConfig::default();
        config.auth.jwt_secret = "test-secret-key".to_string();

        assert!(config.validate().is_ok());

        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_public_path_check() {
        let config = GatewayConfig::default();

        assert!(config.is_public_path("/health"));
        assert!(config.is_public_path("/api/v1/auth/login"));
        assert!(!config.is_public_path("/api/v1/users"));
    }

    #[test]
    fn test_service_endpoint_lookup() {
        let config = GatewayConfig::default();

        assert!(config.get_service_endpoint("user").is_some());
        assert!(config.get_service_endpoint("trading").is_some());
        assert!(config.get_service_endpoint("unknown").is_none());
    }

    #[test]
    fn test_whitelist_check() {
        let config = GatewayConfig::default();

        assert!(config.is_whitelisted_ip("127.0.0.1"));
        assert!(!config.is_whitelisted_ip("192.168.1.1"));
    }
}
