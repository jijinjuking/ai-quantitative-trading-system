use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::Response,
};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::{
    middleware::auth::UserContext,
    middleware::request_id::RequestId,
    services::CircuitBreaker,
    state::AppState,
};

/// 服务代理
#[derive(Clone)]
pub struct ServiceProxy {
    client: Client,
}

impl ServiceProxy {
    /// 创建新的服务代理
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 代理请求到指定服务
    pub async fn proxy_request(
        State(state): State<AppState>,
        Path(params): Path<HashMap<String, String>>,
        Query(query): Query<HashMap<String, String>>,
        request: Request,
    ) -> Result<Response, StatusCode> {
        let service_name = params.get("service")
            .ok_or(StatusCode::BAD_REQUEST)?;

        let path = request.uri().path().to_string();
        let method = request.method().clone();
        
        // 获取请求ID和用户上下文
        let request_id = request.extensions().get::<RequestId>()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let user_context = request.extensions().get::<UserContext>().cloned();

        debug!("Proxying request: {} {} to service: {}", method, path, service_name);

        // 获取服务信息
        let service_info = match state.service_registry.get_healthy_service(service_name).await {
            Some(info) => info,
            None => {
                warn!("Service not available: {}", service_name);
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
        };

        // 获取熔断器
        let circuit_breaker = state.get_circuit_breaker(service_name).await;

        // 检查熔断器状态
        if !circuit_breaker.allow_request().await {
            warn!("Circuit breaker is open for service: {}", service_name);
            if let Err(e) = state.metrics.collector().inc_counter_vec("circuit_breaker_rejections_total", &[service_name]) {
                warn!("Failed to record circuit breaker metrics: {}", e);
            }
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }

        // 构建目标URL
        let target_url = build_target_url(&service_info.url, &path, &query)?;

        // 执行代理请求
        let start_time = Instant::now();
        let result = execute_proxy_request(
            &state,
            &circuit_breaker,
            &target_url,
            method.clone(),
            request,
            &request_id,
            user_context.as_ref(),
        ).await;

        let duration = start_time.elapsed();

        // 记录指标
        match &result {
            Ok(response) => {
                let status = response.status().as_u16();
                let _ = state.metrics.record_http_request(
                    &method.to_string(),
                    &path,
                    status,
                    duration,
                );
                
                info!(
                    request_id = %request_id,
                    service = %service_name,
                    method = %method,
                    path = %path,
                    status = %status,
                    duration_ms = %duration.as_millis(),
                    "Proxy request completed"
                );
            }
            Err(status) => {
                let _ = state.metrics.record_http_request(
                    &method.to_string(),
                    &path,
                    status.as_u16(),
                    duration,
                );
                
                error!(
                    request_id = %request_id,
                    service = %service_name,
                    method = %method,
                    path = %path,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Proxy request failed"
                );
            }
        }

        result
    }

    /// 代理WebSocket连接
    pub async fn proxy_websocket(
        State(state): State<AppState>,
        Path(params): Path<HashMap<String, String>>,
        request: Request,
    ) -> Result<Response, StatusCode> {
        let service_name = params.get("service")
            .ok_or(StatusCode::BAD_REQUEST)?;

        debug!("WebSocket proxy request for service: {}", service_name);

        // 获取服务信息
        let service_info = match state.service_registry.get_healthy_service(service_name).await {
            Some(info) => info,
            None => {
                warn!("WebSocket service not available: {}", service_name);
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
        };

        // 检查WebSocket升级头
        let headers = request.headers();
        if !headers.get("upgrade")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase() == "websocket")
            .unwrap_or(false)
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 构建WebSocket目标URL
        let ws_url = service_info.url.replace("http://", "ws://").replace("https://", "wss://");
        let target_path = request.uri().path().replace(&format!("/ws/{}", service_name), "");
        let target_url = format!("{}{}", ws_url, target_path);

        info!("Proxying WebSocket to: {}", target_url);

        // WebSocket升级需要在路由层处理，这里返回错误
        error!("WebSocket upgrade should be handled at router level");
        Err(StatusCode::BAD_REQUEST)
    }
}

impl Default for ServiceProxy {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行代理请求
async fn execute_proxy_request(
    state: &AppState,
    circuit_breaker: &CircuitBreaker,
    target_url: &str,
    method: Method,
    mut request: Request,
    request_id: &str,
    user_context: Option<&UserContext>,
) -> Result<Response, StatusCode> {
    // 准备请求头
    let mut headers = HeaderMap::new();
    
    // 复制原始请求头（排除某些头部）
    for (name, value) in request.headers() {
        if should_forward_header(name.as_str()) {
            headers.insert(name.clone(), value.clone());
        }
    }

    // 添加代理相关头部
    headers.insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(request_id).unwrap(),
    );

    if let Some(user) = user_context {
        headers.insert(
            HeaderName::from_static("x-user-id"),
            HeaderValue::from_str(&user.user_id).unwrap(),
        );
        headers.insert(
            HeaderName::from_static("x-username"),
            HeaderValue::from_str(&user.username).unwrap(),
        );
    }

    // 获取请求体
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            circuit_breaker.record_failure().await;
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // 创建HTTP客户端请求
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 转换HTTP方法
    let reqwest_method = convert_axum_method_to_reqwest(&method)?;
    
    // 转换HTTP头
    let reqwest_headers = convert_axum_headers_to_reqwest(&headers)?;
    
    let client_request = client
        .request(reqwest_method, target_url)
        .headers(reqwest_headers)
        .body(body_bytes.to_vec());

    let response = match client_request.send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Proxy request failed: {}", e);
            circuit_breaker.record_failure().await;
            
            if e.is_timeout() {
                return Err(StatusCode::GATEWAY_TIMEOUT);
            } else if e.is_connect() {
                return Err(StatusCode::BAD_GATEWAY);
            } else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    // 检查响应状态
    let status = response.status();
    if status.is_server_error() {
        circuit_breaker.record_failure().await;
    } else {
        circuit_breaker.record_success().await;
    }

    // 转换状态码
    let axum_status = convert_reqwest_status_to_axum(status);
    
    // 构建响应
    let mut response_builder = Response::builder().status(axum_status);

    // 转换并复制响应头
    convert_reqwest_headers_to_axum(response.headers(), &mut response_builder)?;

    // 获取响应体
    let response_body = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    match response_builder.body(Body::from(response_body)) {
        Ok(response) => Ok(response),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 构建目标URL
fn build_target_url(
    base_url: &str,
    path: &str,
    query: &HashMap<String, String>,
) -> Result<String, StatusCode> {
    let mut url = format!("{}{}", base_url.trim_end_matches('/'), path);
    
    if !query.is_empty() {
        let query_string: Vec<String> = query
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();
        url.push_str(&format!("?{}", query_string.join("&")));
    }

    Ok(url)
}

/// 检查是否应该转发请求头
fn should_forward_header(header_name: &str) -> bool {
    let skip_headers = [
        "host",
        "connection",
        "upgrade",
        "proxy-connection",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "transfer-encoding",
    ];

    !skip_headers.contains(&header_name.to_lowercase().as_str())
}

/// 检查是否应该转发响应头
fn should_forward_response_header(header_name: &str) -> bool {
    let skip_headers = [
        "connection",
        "upgrade",
        "proxy-connection",
        "proxy-authenticate",
        "te",
        "trailers",
        "transfer-encoding",
    ];

    !skip_headers.contains(&header_name.to_lowercase().as_str())
}

/// 请求重试器
pub struct RequestRetrier {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl RequestRetrier {
    pub fn new(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }

    /// 执行带重试的请求
    pub async fn execute_with_retry<F, Fut, T, E>(
        &self,
        mut operation: F,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.max_retries {
                        let delay = self.calculate_delay(attempt);
                        debug!("Request failed, retrying in {:?} (attempt {})", delay, attempt + 1);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 计算重试延迟（指数退避）
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = self.base_delay * 2_u32.pow(attempt);
        std::cmp::min(delay, self.max_delay)
    }
}

impl Default for RequestRetrier {
    fn default() -> Self {
        Self::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(5),
        )
    }
}

/// 请求转换器
pub struct RequestTransformer;

impl RequestTransformer {
    /// 转换请求路径
    pub fn transform_path(original_path: &str, service_name: &str) -> String {
        // 移除网关前缀
        let gateway_prefix = format!("/api/v1/{}", service_name);
        if original_path.starts_with(&gateway_prefix) {
            original_path.strip_prefix(&gateway_prefix).unwrap_or("/").to_string()
        } else {
            original_path.to_string()
        }
    }

    /// 添加认证头部
    pub fn add_auth_headers(headers: &mut HeaderMap, user_context: &UserContext) {
        if let Ok(user_id) = HeaderValue::from_str(&user_context.user_id) {
            headers.insert(HeaderName::from_static("x-user-id"), user_id);
        }
        
        if let Ok(username) = HeaderValue::from_str(&user_context.username) {
            headers.insert(HeaderName::from_static("x-username"), username);
        }

        // 添加角色和权限信息
        let roles_json = serde_json::to_string(&user_context.roles).unwrap_or_default();
        if let Ok(roles) = HeaderValue::from_str(&roles_json) {
            headers.insert(HeaderName::from_static("x-user-roles"), roles);
        }

        let permissions_json = serde_json::to_string(&user_context.permissions).unwrap_or_default();
        if let Ok(permissions) = HeaderValue::from_str(&permissions_json) {
            headers.insert(HeaderName::from_static("x-user-permissions"), permissions);
        }
    }

    /// 添加跟踪头部
    pub fn add_tracing_headers(headers: &mut HeaderMap, request_id: &str, service_name: &str) {
        if let Ok(req_id) = HeaderValue::from_str(request_id) {
            headers.insert(HeaderName::from_static("x-request-id"), req_id);
        }

        if let Ok(service) = HeaderValue::from_str(service_name) {
            headers.insert(HeaderName::from_static("x-source-service"), HeaderValue::from_static("gateway"));
            headers.insert(HeaderName::from_static("x-target-service"), service);
        }

        // 添加时间戳
        let timestamp = chrono::Utc::now().timestamp().to_string();
        if let Ok(ts) = HeaderValue::from_str(&timestamp) {
            headers.insert(HeaderName::from_static("x-request-timestamp"), ts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_target_url() {
        let base_url = "http://localhost:8080";
        let path = "/api/v1/users";
        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());
        query.insert("limit".to_string(), "10".to_string());

        let url = build_target_url(base_url, path, &query).unwrap();
        assert!(url.starts_with("http://localhost:8080/api/v1/users"));
        assert!(url.contains("page=1"));
        assert!(url.contains("limit=10"));
    }

    #[test]
    fn test_should_forward_header() {
        assert!(should_forward_header("content-type"));
        assert!(should_forward_header("authorization"));
        assert!(!should_forward_header("host"));
        assert!(!should_forward_header("connection"));
    }

    #[test]
    fn test_request_transformer_path() {
        let original = "/api/v1/user/profile";
        let transformed = RequestTransformer::transform_path(original, "user");
        assert_eq!(transformed, "/profile");

        let original = "/health";
        let transformed = RequestTransformer::transform_path(original, "user");
        assert_eq!(transformed, "/health");
    }

    #[tokio::test]
    async fn test_request_retrier() {
        let retrier = RequestRetrier::new(2, Duration::from_millis(10), Duration::from_millis(100));
        
        let mut attempt_count = 0;
        let result = retrier.execute_with_retry(|| {
            attempt_count += 1;
            async move {
                if attempt_count < 2 {
                    Err("temporary failure")
                } else {
                    Ok("success")
                }
            }
        }).await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempt_count, 2);
    }
}

/// 转换Axum HTTP方法到Reqwest HTTP方法
fn convert_axum_method_to_reqwest(method: &axum::http::Method) -> Result<reqwest::Method, StatusCode> {
    match method.as_str() {
        "GET" => Ok(reqwest::Method::GET),
        "POST" => Ok(reqwest::Method::POST),
        "PUT" => Ok(reqwest::Method::PUT),
        "DELETE" => Ok(reqwest::Method::DELETE),
        "HEAD" => Ok(reqwest::Method::HEAD),
        "OPTIONS" => Ok(reqwest::Method::OPTIONS),
        "PATCH" => Ok(reqwest::Method::PATCH),
        "TRACE" => Ok(reqwest::Method::TRACE),
        _ => Err(StatusCode::METHOD_NOT_ALLOWED),
    }
}

/// 转换Axum HeaderMap到Reqwest HeaderMap
fn convert_axum_headers_to_reqwest(headers: &axum::http::HeaderMap) -> Result<reqwest::header::HeaderMap, StatusCode> {
    let mut reqwest_headers = reqwest::header::HeaderMap::new();
    
    for (name, value) in headers {
        // 只转发应该转发的头部
        if should_forward_header(name.as_str()) {
            // 转换头部名称
            let reqwest_name = match reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()) {
                Ok(name) => name,
                Err(_) => continue, // 跳过无效的头部名称
            };
            
            // 转换头部值
            let reqwest_value = match reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                Ok(value) => value,
                Err(_) => continue, // 跳过无效的头部值
            };
            
            reqwest_headers.insert(reqwest_name, reqwest_value);
        }
    }
    
    Ok(reqwest_headers)
}

/// 转换Reqwest响应状态码到Axum状态码
fn convert_reqwest_status_to_axum(status: reqwest::StatusCode) -> axum::http::StatusCode {
    match axum::http::StatusCode::from_u16(status.as_u16()) {
        Ok(status) => status,
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// 转换Reqwest响应头到Axum响应头
fn convert_reqwest_headers_to_axum(
    headers: &reqwest::header::HeaderMap,
    response_builder: &mut axum::http::response::Builder,
) -> Result<(), StatusCode> {
    for (name, value) in headers {
        // 只转发应该转发的响应头
        if should_forward_response_header(name.as_str()) {
            // 转换头部名称
            let axum_name = match axum::http::HeaderName::from_bytes(name.as_str().as_bytes()) {
                Ok(name) => name,
                Err(_) => continue, // 跳过无效的头部名称
            };
            
            // 转换头部值
            let axum_value = match axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                Ok(value) => value,
                Err(_) => continue, // 跳过无效的头部值
            };
            
            *response_builder = std::mem::take(response_builder).header(axum_name, axum_value);
        }
    }
    
    Ok(())
}