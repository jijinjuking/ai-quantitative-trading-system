use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Kafka error: {0}")]
    Kafka(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Configuration error: {0}")]
    Configuration(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Exchange API error: {exchange} - {message}")]
    ExchangeApi { exchange: String, message: String },

    #[error("Trading error: {0}")]
    Trading(String),

    #[error("Risk management error: {0}")]
    RiskManagement(String),

    #[error("Strategy error: {0}")]
    Strategy(String),

    #[error("Market data error: {0}")]
    MarketData(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Business logic error: {0}")]
    BusinessLogic(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Symbol not supported: {0}")]
    SymbolNotSupported(String),

    #[error("Market closed: {0}")]
    MarketClosed(String),

    #[error("Maintenance mode: {0}")]
    MaintenanceMode(String),
}

impl AppError {
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Authentication(_) => "AUTHENTICATION_ERROR",
            AppError::Authorization(_) => "AUTHORIZATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Redis(_) => "REDIS_ERROR",
            AppError::Kafka(_) => "KAFKA_ERROR",
            AppError::HttpClient(_) => "HTTP_CLIENT_ERROR",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::Jwt(_) => "JWT_ERROR",
            AppError::Encryption(_) => "ENCRYPTION_ERROR",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::ExchangeApi { .. } => "EXCHANGE_API_ERROR",
            AppError::Trading(_) => "TRADING_ERROR",
            AppError::RiskManagement(_) => "RISK_MANAGEMENT_ERROR",
            AppError::Strategy(_) => "STRATEGY_ERROR",
            AppError::MarketData(_) => "MARKET_DATA_ERROR",
            AppError::RateLimit(_) => "RATE_LIMIT_ERROR",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::Timeout(_) => "TIMEOUT_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            AppError::BusinessLogic(_) => "BUSINESS_LOGIC_ERROR",
            AppError::InsufficientFunds(_) => "INSUFFICIENT_FUNDS",
            AppError::InvalidOrder(_) => "INVALID_ORDER",
            AppError::PositionNotFound(_) => "POSITION_NOT_FOUND",
            AppError::SymbolNotSupported(_) => "SYMBOL_NOT_SUPPORTED",
            AppError::MarketClosed(_) => "MARKET_CLOSED",
            AppError::MaintenanceMode(_) => "MAINTENANCE_MODE",
        }
    }

    /// 获取HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Timeout(_) => StatusCode::REQUEST_TIMEOUT,
            AppError::InsufficientFunds(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidOrder(_) => StatusCode::BAD_REQUEST,
            AppError::PositionNotFound(_) => StatusCode::NOT_FOUND,
            AppError::SymbolNotSupported(_) => StatusCode::BAD_REQUEST,
            AppError::MarketClosed(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::MaintenanceMode(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 是否应该记录错误
    pub fn should_log(&self) -> bool {
        match self {
            AppError::Validation(_) => false,
            AppError::Authentication(_) => false,
            AppError::Authorization(_) => false,
            AppError::NotFound(_) => false,
            AppError::Conflict(_) => false,
            AppError::RateLimit(_) => false,
            AppError::InsufficientFunds(_) => false,
            AppError::InvalidOrder(_) => false,
            AppError::PositionNotFound(_) => false,
            AppError::SymbolNotSupported(_) => false,
            AppError::MarketClosed(_) => false,
            _ => true,
        }
    }

    /// 是否应该显示详细错误信息
    pub fn should_show_details(&self) -> bool {
        match self {
            AppError::Database(_) => false,
            AppError::Redis(_) => false,
            AppError::Kafka(_) => false,
            AppError::Internal(_) => false,
            AppError::Configuration(_) => false,
            AppError::Io(_) => false,
            _ => true,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();

        // 记录错误
        if self.should_log() {
            tracing::error!(
                error = %self,
                error_code = error_code,
                status = %status,
                "Application error occurred"
            );
        }

        // 构建响应
        let message = if self.should_show_details() {
            self.to_string()
        } else {
            "Internal server error".to_string()
        };

        let body = json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        (status, Json(body)).into_response()
    }
}

/// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;

/// 错误处理工具
pub struct ErrorHandler;

impl ErrorHandler {
    /// 包装数据库错误
    pub fn database_error(err: sqlx::Error) -> AppError {
        match &err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    AppError::Conflict("Resource already exists".to_string())
                } else if db_err.is_foreign_key_violation() {
                    AppError::Validation("Invalid reference".to_string())
                } else {
                    AppError::Database(err)
                }
            }
            _ => AppError::Database(err),
        }
    }

    /// 包装Redis错误
    pub fn redis_error(err: redis::RedisError) -> AppError {
        AppError::Redis(err)
    }

    /// 包装HTTP客户端错误
    pub fn http_client_error(err: reqwest::Error) -> AppError {
        if err.is_timeout() {
            AppError::Timeout("Request timeout".to_string())
        } else if err.is_connect() {
            AppError::ServiceUnavailable("Service unavailable".to_string())
        } else {
            AppError::HttpClient(err)
        }
    }

    /// 创建验证错误
    pub fn validation_error(message: &str) -> AppError {
        AppError::Validation(message.to_string())
    }

    /// 创建认证错误
    pub fn authentication_error(message: &str) -> AppError {
        AppError::Authentication(message.to_string())
    }

    /// 创建授权错误
    pub fn authorization_error(message: &str) -> AppError {
        AppError::Authorization(message.to_string())
    }

    /// 创建未找到错误
    pub fn not_found_error(resource: &str) -> AppError {
        AppError::NotFound(format!("{} not found", resource))
    }

    /// 创建冲突错误
    pub fn conflict_error(message: &str) -> AppError {
        AppError::Conflict(message.to_string())
    }

    /// 创建业务逻辑错误
    pub fn business_logic_error(message: &str) -> AppError {
        AppError::BusinessLogic(message.to_string())
    }

    /// 创建交易错误
    pub fn trading_error(message: &str) -> AppError {
        AppError::Trading(message.to_string())
    }

    /// 创建风险管理错误
    pub fn risk_management_error(message: &str) -> AppError {
        AppError::RiskManagement(message.to_string())
    }

    /// 创建策略错误
    pub fn strategy_error(message: &str) -> AppError {
        AppError::Strategy(message.to_string())
    }

    /// 创建市场数据错误
    pub fn market_data_error(message: &str) -> AppError {
        AppError::MarketData(message.to_string())
    }

    /// 创建交易所API错误
    pub fn exchange_api_error(exchange: &str, message: &str) -> AppError {
        AppError::ExchangeApi {
            exchange: exchange.to_string(),
            message: message.to_string(),
        }
    }

    /// 创建外部服务错误
    pub fn external_service_error(service: &str, message: &str) -> AppError {
        AppError::ExternalService {
            service: service.to_string(),
            message: message.to_string(),
        }
    }
}

/// 错误转换宏
#[macro_export]
macro_rules! app_error {
    ($variant:ident, $msg:expr) => {
        AppError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        AppError::$variant(format!($fmt, $($arg)*))
    };
}

/// 验证错误宏
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        AppError::Validation($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        AppError::Validation(format!($fmt, $($arg)*))
    };
}

/// 业务逻辑错误宏
#[macro_export]
macro_rules! business_error {
    ($msg:expr) => {
        AppError::BusinessLogic($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        AppError::BusinessLogic(format!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let validation_error = AppError::Validation("test".to_string());
        assert_eq!(validation_error.error_code(), "VALIDATION_ERROR");
        assert_eq!(validation_error.status_code(), StatusCode::BAD_REQUEST);
        assert!(!validation_error.should_log());
        assert!(validation_error.should_show_details());
    }

    #[test]
    fn test_error_handler() {
        let validation_error = ErrorHandler::validation_error("Invalid input");
        assert_eq!(validation_error.error_code(), "VALIDATION_ERROR");

        let not_found_error = ErrorHandler::not_found_error("User");
        assert_eq!(not_found_error.error_code(), "NOT_FOUND");
        assert_eq!(not_found_error.to_string(), "Not found: User not found");
    }

    #[test]
    fn test_error_macros() {
        let error = app_error!(Validation, "test message");
        assert_eq!(error.error_code(), "VALIDATION_ERROR");

        let error = validation_error!("Invalid field: {}", "email");
        assert_eq!(error.to_string(), "Validation error: Invalid field: email");

        let error = business_error!("Insufficient balance");
        assert_eq!(error.error_code(), "BUSINESS_LOGIC_ERROR");
    }
}
