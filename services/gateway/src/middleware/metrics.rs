use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::info;

use crate::state::AppState;

/// 指标中间件
#[derive(Clone)]
pub struct MetricsMiddleware {
    state: AppState,
}

impl MetricsMiddleware {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

/// 指标中间件处理函数
pub async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    
    // 获取请求ID（如果存在）
    let request_id = request.extensions()
        .get::<crate::middleware::request_id::RequestId>()
        .map(|id| id.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 执行请求
    let response = next.run(request).await;
    
    // 计算响应时间
    let duration = start_time.elapsed();
    let status_code = response.status().as_u16();
    
    // 记录HTTP请求指标
    let _ = state.metrics.record_http_request(
        &method,
        &path,
        status_code,
        duration,
    );

    // 记录访问日志
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = %status_code,
        duration_ms = %duration.as_millis(),
        "HTTP request completed"
    );

    response
}

/// 业务指标记录器
pub struct BusinessMetrics {
    state: AppState,
}

impl BusinessMetrics {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// 记录用户登录
    pub async fn record_user_login(&self, user_id: &str, success: bool) {
        let status = if success { "success" } else { "failure" };
        let _ = self.state.metrics.collector()
            .inc_counter_vec("user_login_total", &[status]);
        
        info!(
            user_id = %user_id,
            success = %success,
            "User login attempt"
        );
    }

    /// 记录API调用
    pub async fn record_api_call(&self, service: &str, method: &str, success: bool, duration_ms: u64) {
        let status = if success { "success" } else { "failure" };
        let _ = self.state.metrics.collector()
            .inc_counter_vec("api_calls_total", &[service, method, status]);
        
        let _ = self.state.metrics.collector()
            .observe_histogram_vec("api_call_duration_seconds", &[service, method], duration_ms as f64 / 1000.0);
        
        info!(
            service = %service,
            method = %method,
            success = %success,
            duration_ms = %duration_ms,
            "API call completed"
        );
    }

    /// 记录错误
    pub async fn record_error(&self, error_type: &str, service: Option<&str>) {
        let service_name = service.unwrap_or("gateway");
        let _ = self.state.metrics.collector()
            .inc_counter_vec("errors_total", &[error_type, service_name]);
        
        info!(
            error_type = %error_type,
            service = %service_name,
            "Error recorded"
        );
    }

    /// 记录缓存命中/未命中
    pub async fn record_cache_hit(&self, cache_type: &str, hit: bool) {
        let status = if hit { "hit" } else { "miss" };
        let _ = self.state.metrics.collector()
            .inc_counter_vec("cache_requests_total", &[cache_type, status]);
    }

    /// 记录限流事件
    pub async fn record_rate_limit(&self, client_type: &str, action: &str) {
        let _ = self.state.metrics.collector()
            .inc_counter_vec("rate_limit_events_total", &[client_type, action]);
    }

    /// 记录熔断器事件
    pub async fn record_circuit_breaker(&self, service: &str, state: &str) {
        let _ = self.state.metrics.collector()
            .inc_counter_vec("circuit_breaker_events_total", &[service, state]);
        
        info!(
            service = %service,
            state = %state,
            "Circuit breaker state changed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_duration_calculation() {
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(10));
        let duration = start.elapsed();
        
        assert!(duration.as_millis() >= 10);
    }
}