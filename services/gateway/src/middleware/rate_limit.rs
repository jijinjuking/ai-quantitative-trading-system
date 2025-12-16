use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use tracing::{debug, warn};

use crate::state::AppState;

/// 限流中间件
#[derive(Clone)]
pub struct RateLimitMiddleware {
    state: AppState,
}

impl RateLimitMiddleware {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

/// 限流中间件处理函数
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 检查限流是否启用
    if !state.config.rate_limit.enabled {
        return Ok(next.run(request).await);
    }

    let client_ip = addr.ip().to_string();
    
    // 检查IP是否在白名单中
    if state.config.is_whitelisted_ip(&client_ip) {
        debug!("Whitelisted IP accessed: {}", client_ip);
        return Ok(next.run(request).await);
    }

    // 获取用户ID（如果已认证）
    let user_id = request.extensions()
        .get::<crate::middleware::auth::UserContext>()
        .map(|ctx| ctx.user_id.clone());

    // 构建限流键
    let rate_limit_key = match user_id {
        Some(uid) => format!("user:{}", uid),
        None => format!("ip:{}", client_ip),
    };

    // 检查限流
    match state.rate_limiter.check_rate_limit(&rate_limit_key).await {
        Ok(allowed) => {
            if allowed {
                debug!("Rate limit check passed for: {}", rate_limit_key);
                
                // 记录成功的请求
                if let Err(e) = state.metrics.collector().inc_counter_vec("rate_limit_requests_total", &["allowed"]) {
                    warn!("Failed to record rate limit metrics: {}", e);
                }
                
                Ok(next.run(request).await)
            } else {
                warn!("Rate limit exceeded for: {}", rate_limit_key);
                
                // 记录被限流的请求
                if let Err(e) = state.metrics.collector().inc_counter_vec("rate_limit_requests_total", &["rejected"]) {
                    warn!("Failed to record rate limit metrics: {}", e);
                }
                
                Err(StatusCode::TOO_MANY_REQUESTS)
            }
        }
        Err(e) => {
            warn!("Rate limit check failed: {}", e);
            
            // 记录错误
            if let Err(e) = state.metrics.collector().inc_counter_vec("rate_limit_requests_total", &["error"]) {
                warn!("Failed to record rate limit metrics: {}", e);
            }
            
            // 限流检查失败时，允许请求通过（fail-open策略）
            Ok(next.run(request).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_rate_limit_key_generation() {
        // 测试IP限流键
        let ip = "192.168.1.1";
        let key = format!("ip:{}", ip);
        assert_eq!(key, "ip:192.168.1.1");

        // 测试用户限流键
        let user_id = "user123";
        let key = format!("user:{}", user_id);
        assert_eq!(key, "user:user123");
    }
}