use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use shared_protocols::http::ApiError;
use tracing::error;

/// 全局错误处理器
pub async fn handle_error(err: Box<dyn std::error::Error + Send + Sync>) -> Response {
    error!("Unhandled error: {}", err);
    
    let api_error = ApiError::new("INTERNAL_ERROR", "Internal server error");
    let response = json!({
        "success": false,
        "error": api_error,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
}

/// 404处理器
pub async fn handle_404() -> Response {
    let api_error = ApiError::new("NOT_FOUND", "Endpoint not found");
    let response = json!({
        "success": false,
        "error": api_error,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    (StatusCode::NOT_FOUND, Json(response)).into_response()
}

/// 方法不允许处理器
pub async fn handle_405() -> Response {
    let api_error = ApiError::new("METHOD_NOT_ALLOWED", "Method not allowed");
    let response = json!({
        "success": false,
        "error": api_error,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    (StatusCode::METHOD_NOT_ALLOWED, Json(response)).into_response()
}