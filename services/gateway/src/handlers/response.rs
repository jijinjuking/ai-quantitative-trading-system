use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use shared_protocols::http::ApiResponse;

/// 响应构建器
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
}

impl ResponseBuilder {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
        }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            self.headers.insert(name, value);
        }
        self
    }

    pub fn json<T: serde::Serialize>(self, data: T) -> Response {
        let body = Json(data);
        (self.status, self.headers, body).into_response()
    }

    pub fn text(self, text: String) -> Response {
        (self.status, self.headers, text).into_response()
    }
}

/// 成功响应
pub fn success_response<T: serde::Serialize>(data: T) -> Response {
    ResponseBuilder::new(StatusCode::OK)
        .header("content-type", "application/json")
        .json(ApiResponse::success(data))
}

/// 错误响应
pub fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    let error = shared_protocols::http::ApiError::new(code, message);
    ResponseBuilder::new(status)
        .header("content-type", "application/json")
        .json(ApiResponse::<()>::error(error))
}

/// 验证错误响应
pub fn validation_error_response(message: &str) -> Response {
    error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message)
}

/// 认证错误响应
pub fn auth_error_response() -> Response {
    error_response(
        StatusCode::UNAUTHORIZED,
        "AUTHENTICATION_ERROR",
        "Authentication required",
    )
}

/// 授权错误响应
pub fn forbidden_response() -> Response {
    error_response(
        StatusCode::FORBIDDEN,
        "AUTHORIZATION_ERROR",
        "Insufficient permissions",
    )
}

/// 未找到错误响应
pub fn not_found_response(resource: &str) -> Response {
    error_response(
        StatusCode::NOT_FOUND,
        "NOT_FOUND",
        &format!("{} not found", resource),
    )
}

/// 限流错误响应
pub fn rate_limit_response() -> Response {
    ResponseBuilder::new(StatusCode::TOO_MANY_REQUESTS)
        .header("content-type", "application/json")
        .header("retry-after", "60")
        .json(ApiResponse::<()>::error(
            shared_protocols::http::ApiError::new("RATE_LIMIT_EXCEEDED", "Rate limit exceeded"),
        ))
}

/// 服务不可用响应
pub fn service_unavailable_response() -> Response {
    error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        "Service temporarily unavailable",
    )
}
