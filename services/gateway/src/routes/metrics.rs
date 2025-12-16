use axum::{extract::State, http::StatusCode, response::Response};

use crate::state::AppState;

/// 指标处理器
pub async fn metrics_handler(State(state): State<AppState>) -> Result<Response, StatusCode> {
    match state.metrics.gather() {
        Ok(metrics_text) => {
            Response::builder()
                .status(200)
                .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
                .body(axum::body::Body::from(metrics_text))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}