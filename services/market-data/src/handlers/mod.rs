pub mod health;
pub mod market_data;
pub mod metrics;
pub mod websocket;

use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub use health::health_handler;
pub use market_data::{
    get_exchanges, get_latest_kline, get_latest_orderbook, get_latest_tick, get_latest_trade,
    get_symbols,
};
pub use metrics::metrics_handler;
pub use websocket::websocket_handler;

/// 创建所有路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 健康检查
        .route("/health", get(health_handler))
        .route("/health/detailed", get(health::detailed_health_handler))
        // 指标
        .route("/metrics", get(metrics_handler))
        // WebSocket连接
        .route("/ws", get(websocket_handler))
        // 市场数据API
        .route("/api/v1/tick/:exchange/:symbol", get(get_latest_tick))
        .route(
            "/api/v1/kline/:exchange/:symbol/:interval",
            get(get_latest_kline),
        )
        .route(
            "/api/v1/orderbook/:exchange/:symbol",
            get(get_latest_orderbook),
        )
        .route("/api/v1/trade/:exchange/:symbol", get(get_latest_trade))
        // 元数据API
        .route("/api/v1/symbols", get(get_symbols))
        .route("/api/v1/exchanges", get(get_exchanges))
        // 管理API
        .route("/api/v1/admin/stats", get(market_data::get_stats))
        .route("/api/v1/admin/flush", post(market_data::flush_buffers))
        .route("/api/v1/admin/reset-stats", post(market_data::reset_stats))
}

/// API响应结构
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: i64,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// 创建错误响应
    pub fn error(error: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// API错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Unauthorized")]
    Unauthorized,
}

impl ApiError {
    /// 转换为HTTP状态码
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            ApiError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            ApiError::InternalServerError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            ApiError::RateLimitExceeded => axum::http::StatusCode::TOO_MANY_REQUESTS,
            ApiError::Unauthorized => axum::http::StatusCode::UNAUTHORIZED,
        }
    }

    /// 转换为API响应
    pub fn to_response(&self) -> (axum::http::StatusCode, axum::Json<ApiResponse<()>>) {
        let status = self.status_code();
        let response = ApiResponse::error(self.to_string());
        (status, axum::Json(response))
    }
}

/// 实现从anyhow::Error的转换
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalServerError(err.to_string())
    }
}

/// 实现Axum的IntoResponse trait
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, json) = self.to_response();
        (status, json).into_response()
    }
}

/// 分页参数
#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(50),
        }
    }
}

impl PaginationParams {
    /// 获取偏移量
    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1);
        let limit = self.limit.unwrap_or(50);
        (page.saturating_sub(1)) * limit
    }

    /// 获取限制数量
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(1000) // 最大1000条
    }

    /// 验证参数
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(page) = self.page {
            if page == 0 {
                return Err(ApiError::BadRequest(
                    "Page must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(limit) = self.limit {
            if limit == 0 {
                return Err(ApiError::BadRequest(
                    "Limit must be greater than 0".to_string(),
                ));
            }
            if limit > 1000 {
                return Err(ApiError::BadRequest("Limit cannot exceed 1000".to_string()));
            }
        }

        Ok(())
    }
}

/// 时间范围参数
#[derive(Debug, serde::Deserialize)]
pub struct TimeRangeParams {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

impl TimeRangeParams {
    /// 验证时间范围
    pub fn validate(&self) -> Result<(), ApiError> {
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            if start >= end {
                return Err(ApiError::BadRequest(
                    "Start time must be before end time".to_string(),
                ));
            }

            let now = chrono::Utc::now().timestamp_millis();
            if start > now || end > now {
                return Err(ApiError::BadRequest(
                    "Time cannot be in the future".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 获取开始时间（默认为24小时前）
    pub fn start_time(&self) -> i64 {
        self.start_time.unwrap_or_else(|| {
            chrono::Utc::now().timestamp_millis() - 24 * 60 * 60 * 1000 // 24小时前
        })
    }

    /// 获取结束时间（默认为当前时间）
    pub fn end_time(&self) -> i64 {
        self.end_time
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis())
    }
}

/// 通用查询参数
#[derive(Debug, serde::Deserialize)]
pub struct QueryParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub time_range: TimeRangeParams,
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub interval: Option<String>,
}

impl QueryParams {
    /// 验证所有参数
    pub fn validate(&self) -> Result<(), ApiError> {
        self.pagination.validate()?;
        self.time_range.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let success_response = ApiResponse::success("test data");
        assert!(success_response.success);
        assert_eq!(success_response.data, Some("test data"));
        assert!(success_response.error.is_none());

        let error_response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert_eq!(error_response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams {
            page: Some(2),
            limit: Some(25),
        };

        assert_eq!(params.offset(), 25);
        assert_eq!(params.limit(), 25);
        assert!(params.validate().is_ok());

        let invalid_params = PaginationParams {
            page: Some(0),
            limit: Some(25),
        };
        assert!(invalid_params.validate().is_err());
    }

    #[test]
    fn test_time_range_params() {
        let now = chrono::Utc::now().timestamp_millis();
        let params = TimeRangeParams {
            start_time: Some(now - 3600000), // 1小时前
            end_time: Some(now),
        };

        assert!(params.validate().is_ok());
        assert_eq!(params.start_time(), now - 3600000);
        assert_eq!(params.end_time(), now);

        let invalid_params = TimeRangeParams {
            start_time: Some(now),
            end_time: Some(now - 3600000), // 结束时间在开始时间之前
        };
        assert!(invalid_params.validate().is_err());
    }

    #[test]
    fn test_api_error() {
        let error = ApiError::NotFound("Resource not found".to_string());
        assert_eq!(error.status_code(), axum::http::StatusCode::NOT_FOUND);

        let (status, _) = error.to_response();
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    }
}
