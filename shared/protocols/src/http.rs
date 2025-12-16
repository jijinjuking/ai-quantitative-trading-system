use anyhow::Result;
use serde::{Deserialize, Serialize};
use shared_models::*;
use std::collections::HashMap;

/// HTTP API响应包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    pub fn success_with_request_id(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            request_id: Some(request_id),
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    pub fn error_with_request_id(error: ApiError, request_id: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            request_id: Some(request_id),
        }
    }
}

/// API错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(code: &str, message: &str, details: serde_json::Value) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }

    // 常用错误
    pub fn validation_error(message: &str) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn authentication_error() -> Self {
        Self::new("AUTHENTICATION_ERROR", "Authentication required")
    }

    pub fn authorization_error() -> Self {
        Self::new("AUTHORIZATION_ERROR", "Insufficient permissions")
    }

    pub fn not_found(resource: &str) -> Self {
        Self::new("NOT_FOUND", &format!("{} not found", resource))
    }

    pub fn internal_error() -> Self {
        Self::new("INTERNAL_ERROR", "Internal server error")
    }

    pub fn rate_limit_exceeded() -> Self {
        Self::new("RATE_LIMIT_EXCEEDED", "Rate limit exceeded")
    }

    pub fn service_unavailable() -> Self {
        Self::new("SERVICE_UNAVAILABLE", "Service temporarily unavailable")
    }
}

/// 分页请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationRequest {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            sort_by: None,
            sort_order: Some(SortOrder::Desc),
        }
    }
}

/// 排序顺序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationInfo,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(current_page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;
        let has_next = current_page < total_pages;
        let has_prev = current_page > 1;

        Self {
            current_page,
            page_size,
            total_items,
            total_pages,
            has_next,
            has_prev,
        }
    }
}

/// HTTP请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub headers: HashMap<String, String>,
}

impl RequestContext {
    pub fn new(request_id: String, ip_address: String) -> Self {
        Self {
            request_id,
            user_id: None,
            ip_address,
            user_agent: None,
            timestamp: chrono::Utc::now(),
            headers: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
}

/// API版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiVersion {
    V1,
    V2,
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
        }
    }
}

/// REST API端点定义
pub struct ApiEndpoints;

impl ApiEndpoints {
    // 用户管理
    pub const USERS: &'static str = "/api/v1/users";
    pub const USER_BY_ID: &'static str = "/api/v1/users/{id}";
    pub const USER_LOGIN: &'static str = "/api/v1/auth/login";
    pub const USER_LOGOUT: &'static str = "/api/v1/auth/logout";
    pub const USER_REFRESH: &'static str = "/api/v1/auth/refresh";

    // 交易
    pub const ORDERS: &'static str = "/api/v1/orders";
    pub const ORDER_BY_ID: &'static str = "/api/v1/orders/{id}";
    pub const TRADES: &'static str = "/api/v1/trades";
    pub const POSITIONS: &'static str = "/api/v1/positions";
    pub const BALANCES: &'static str = "/api/v1/balances";

    // 市场数据
    pub const MARKET_TICKERS: &'static str = "/api/v1/market/tickers";
    pub const MARKET_KLINES: &'static str = "/api/v1/market/klines";
    pub const MARKET_ORDERBOOK: &'static str = "/api/v1/market/orderbook";
    pub const MARKET_TRADES: &'static str = "/api/v1/market/trades";

    // 策略
    pub const STRATEGIES: &'static str = "/api/v1/strategies";
    pub const STRATEGY_BY_ID: &'static str = "/api/v1/strategies/{id}";
    pub const BACKTESTS: &'static str = "/api/v1/backtests";
    pub const BACKTEST_BY_ID: &'static str = "/api/v1/backtests/{id}";

    // 风险管理
    pub const RISK_RULES: &'static str = "/api/v1/risk/rules";
    pub const RISK_LIMITS: &'static str = "/api/v1/risk/limits";
    pub const RISK_REPORTS: &'static str = "/api/v1/risk/reports";

    // 系统
    pub const HEALTH: &'static str = "/health";
    pub const METRICS: &'static str = "/metrics";
    pub const VERSION: &'static str = "/version";
}

/// HTTP状态码常量
pub struct HttpStatus;

impl HttpStatus {
    pub const OK: u16 = 200;
    pub const CREATED: u16 = 201;
    pub const ACCEPTED: u16 = 202;
    pub const NO_CONTENT: u16 = 204;

    pub const BAD_REQUEST: u16 = 400;
    pub const UNAUTHORIZED: u16 = 401;
    pub const FORBIDDEN: u16 = 403;
    pub const NOT_FOUND: u16 = 404;
    pub const METHOD_NOT_ALLOWED: u16 = 405;
    pub const CONFLICT: u16 = 409;
    pub const UNPROCESSABLE_ENTITY: u16 = 422;
    pub const TOO_MANY_REQUESTS: u16 = 429;

    pub const INTERNAL_SERVER_ERROR: u16 = 500;
    pub const BAD_GATEWAY: u16 = 502;
    pub const SERVICE_UNAVAILABLE: u16 = 503;
    pub const GATEWAY_TIMEOUT: u16 = 504;
}

/// HTTP头部常量
pub struct HttpHeaders;

impl HttpHeaders {
    pub const AUTHORIZATION: &'static str = "Authorization";
    pub const CONTENT_TYPE: &'static str = "Content-Type";
    pub const ACCEPT: &'static str = "Accept";
    pub const USER_AGENT: &'static str = "User-Agent";
    pub const X_REQUEST_ID: &'static str = "X-Request-ID";
    pub const X_API_KEY: &'static str = "X-API-Key";
    pub const X_SIGNATURE: &'static str = "X-Signature";
    pub const X_TIMESTAMP: &'static str = "X-Timestamp";
    pub const X_RATE_LIMIT_REMAINING: &'static str = "X-RateLimit-Remaining";
    pub const X_RATE_LIMIT_RESET: &'static str = "X-RateLimit-Reset";
}

/// 内容类型常量
pub struct ContentTypes;

impl ContentTypes {
    pub const JSON: &'static str = "application/json";
    pub const FORM_URLENCODED: &'static str = "application/x-www-form-urlencoded";
    pub const MULTIPART_FORM_DATA: &'static str = "multipart/form-data";
    pub const TEXT_PLAIN: &'static str = "text/plain";
    pub const TEXT_HTML: &'static str = "text/html";
}

/// API请求构建器
pub struct ApiRequestBuilder {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
    body: Option<serde_json::Value>,
}

impl ApiRequestBuilder {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.query_params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn json_body<T: Serialize>(mut self, body: &T) -> Result<Self> {
        self.body = Some(serde_json::to_value(body)?);
        self.headers.insert(
            HttpHeaders::CONTENT_TYPE.to_string(),
            ContentTypes::JSON.to_string(),
        );
        Ok(self)
    }

    pub fn auth_bearer(mut self, token: &str) -> Self {
        self.headers.insert(
            HttpHeaders::AUTHORIZATION.to_string(),
            format!("Bearer {}", token),
        );
        self
    }

    pub fn auth_api_key(mut self, api_key: &str) -> Self {
        self.headers
            .insert(HttpHeaders::X_API_KEY.to_string(), api_key.to_string());
        self
    }

    pub fn build(self) -> ApiRequest {
        ApiRequest {
            method: self.method,
            path: self.path,
            headers: self.headers,
            query_params: self.query_params,
            body: self.body,
        }
    }
}

/// API请求
#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

impl ApiRequest {
    pub fn get(path: &str) -> ApiRequestBuilder {
        ApiRequestBuilder::new("GET", path)
    }

    pub fn post(path: &str) -> ApiRequestBuilder {
        ApiRequestBuilder::new("POST", path)
    }

    pub fn put(path: &str) -> ApiRequestBuilder {
        ApiRequestBuilder::new("PUT", path)
    }

    pub fn delete(path: &str) -> ApiRequestBuilder {
        ApiRequestBuilder::new("DELETE", path)
    }

    pub fn patch(path: &str) -> ApiRequestBuilder {
        ApiRequestBuilder::new("PATCH", path)
    }

    /// 构建完整URL
    pub fn build_url(&self, base_url: &str) -> String {
        let mut url = format!("{}{}", base_url.trim_end_matches('/'), self.path);

        if !self.query_params.is_empty() {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            url.push_str(&format!("?{}", query_string.join("&")));
        }

        url
    }
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime: u64,
    pub checks: HashMap<String, ServiceHealth>,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

/// 服务健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// 版本信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
    pub build_time: String,
    pub git_commit: String,
    pub rust_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());

        let error_response =
            ApiResponse::<String>::error(ApiError::validation_error("Invalid input"));
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo::new(2, 10, 25);
        assert_eq!(pagination.current_page, 2);
        assert_eq!(pagination.page_size, 10);
        assert_eq!(pagination.total_items, 25);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }

    #[test]
    fn test_api_request_builder() {
        let request = ApiRequest::get("/api/v1/users")
            .header("Authorization", "Bearer token123")
            .query_param("page", "1")
            .query_param("limit", "10")
            .build();

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/api/v1/users");
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(request.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(request.query_params.get("limit"), Some(&"10".to_string()));

        let url = request.build_url("https://api.example.com");
        assert!(url.contains("https://api.example.com/api/v1/users"));
        assert!(url.contains("page=1"));
        assert!(url.contains("limit=10"));
    }

    #[test]
    fn test_api_error() {
        let error = ApiError::validation_error("Field is required");
        assert_eq!(error.code, "VALIDATION_ERROR");
        assert_eq!(error.message, "Field is required");

        let error = ApiError::not_found("User");
        assert_eq!(error.code, "NOT_FOUND");
        assert_eq!(error.message, "User not found");
    }
}
