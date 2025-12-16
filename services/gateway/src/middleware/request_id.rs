use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// 请求ID中间件
#[derive(Clone)]
pub struct RequestIdMiddleware;

impl RequestIdMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequestIdMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求ID中间件处理函数
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // 检查是否已有请求ID
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 将请求ID添加到请求扩展中
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // 执行下一个中间件/处理器
    let mut response = next.run(request).await;

    // 将请求ID添加到响应头中
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(
            HeaderName::from_static("x-request-id"),
            header_value,
        );
    }

    response
}

/// 请求ID包装器
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 从请求中提取请求ID的辅助函数
pub fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

/// 生成新的请求ID
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_request_id() {
        let mut headers = HeaderMap::new();
        
        // 测试没有请求ID的情况
        assert_eq!(extract_request_id(&headers), None);
        
        // 测试有请求ID的情况
        let request_id = "test-request-id-123";
        headers.insert("x-request-id", HeaderValue::from_static(request_id));
        assert_eq!(extract_request_id(&headers), Some(request_id.to_string()));
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        
        // 确保生成的ID不同
        assert_ne!(id1, id2);
        
        // 确保ID格式正确（UUID格式）
        assert!(Uuid::parse_str(&id1).is_ok());
        assert!(Uuid::parse_str(&id2).is_ok());
    }

    #[test]
    fn test_request_id_wrapper() {
        let id = "test-id-123";
        let request_id = RequestId(id.to_string());
        
        assert_eq!(request_id.as_str(), id);
        assert_eq!(request_id.to_string(), id);
    }
}