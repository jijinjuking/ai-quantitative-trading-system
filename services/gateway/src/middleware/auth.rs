use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use shared_protocols::http::{ApiError, ApiResponse};
use shared_utils::Claims;
use tracing::{debug, warn};

use crate::state::AppState;

/// 认证中间件
#[derive(Clone)]
pub struct AuthMiddleware {
    state: AppState,
}

impl AuthMiddleware {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

/// 认证中间件处理函数
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // 检查是否为公开路径
    if state.config.is_public_path(path) {
        debug!("Public path accessed: {}", path);
        return Ok(next.run(request).await);
    }

    // 提取Authorization头
    let headers = request.headers();
    let auth_header = match extract_auth_header(headers) {
        Some(header) => header,
        None => {
            warn!("Missing authorization header for path: {}", path);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // 验证JWT token
    let claims = match state.jwt_service.verify_token(&auth_header) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Invalid JWT token: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // 将用户信息添加到请求扩展中
    request.extensions_mut().insert(UserContext {
        user_id: claims.sub.clone(),
        username: claims.username.clone(),
        email: claims.email.clone(),
        roles: claims.roles.clone(),
        permissions: claims.permissions.clone(),
    });

    debug!("User authenticated: {} ({})", claims.username, claims.sub);

    // 记录认证指标
    if let Err(e) = state.metrics.collector().inc_counter_vec("auth_requests_total", &["success"]) {
        warn!("Failed to record auth metrics: {}", e);
    }

    Ok(next.run(request).await)
}

/// 提取Authorization头
fn extract_auth_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(value[7..].to_string())
            } else {
                None
            }
        })
}

/// 用户上下文
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl UserContext {
    /// 检查是否有指定权限
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// 检查是否有指定角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// 检查是否有任意一个权限
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    /// 检查是否有所有权限
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }
}

/// 权限检查中间件
pub async fn permission_middleware(
    required_permission: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let permission = required_permission;
        Box::pin(async move {
            // 从请求扩展中获取用户上下文
            let user_context = request.extensions().get::<UserContext>();
            
            match user_context {
                Some(context) => {
                    if context.has_permission(permission) {
                        Ok(next.run(request).await)
                    } else {
                        warn!("Permission denied: user {} lacks permission {}", context.user_id, permission);
                        Err(StatusCode::FORBIDDEN)
                    }
                }
                None => {
                    warn!("No user context found in request");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        })
    }
}

/// 角色检查中间件
pub async fn role_middleware(
    required_role: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let role = required_role;
        Box::pin(async move {
            // 从请求扩展中获取用户上下文
            let user_context = request.extensions().get::<UserContext>();
            
            match user_context {
                Some(context) => {
                    if context.has_role(role) {
                        Ok(next.run(request).await)
                    } else {
                        warn!("Role check failed: user {} lacks role {}", context.user_id, role);
                        Err(StatusCode::FORBIDDEN)
                    }
                }
                None => {
                    warn!("No user context found in request");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_auth_header() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer token123"));
        
        let token = extract_auth_header(&headers);
        assert_eq!(token, Some("token123".to_string()));

        // 测试无效格式
        headers.insert("authorization", HeaderValue::from_static("Invalid token123"));
        let token = extract_auth_header(&headers);
        assert_eq!(token, None);

        // 测试缺失头部
        headers.remove("authorization");
        let token = extract_auth_header(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_user_context_permissions() {
        let context = UserContext {
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["admin".to_string(), "trader".to_string()],
            permissions: vec!["read".to_string(), "write".to_string(), "trade".to_string()],
        };

        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
        assert!(!context.has_permission("admin"));

        assert!(context.has_role("admin"));
        assert!(context.has_role("trader"));
        assert!(!context.has_role("viewer"));

        assert!(context.has_any_permission(&["read", "admin"]));
        assert!(!context.has_any_permission(&["admin", "delete"]));

        assert!(context.has_all_permissions(&["read", "write"]));
        assert!(!context.has_all_permissions(&["read", "admin"]));
    }
}
// UserContext可以通过Option<Extension<UserContext>>来提取