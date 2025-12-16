use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use shared_protocols::http::{ApiError, ApiResponse};
use shared_utils::Claims;
use tracing::{info, warn};

use crate::{middleware::auth::UserContext, state::AppState};

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

/// 刷新令牌请求
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 登录处理器
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    // 这里应该调用用户服务进行认证
    // 为了演示，我们创建一个模拟的成功响应
    
    info!("Login attempt for user: {}", request.username);
    
    // 模拟用户验证（实际应该调用用户服务）
    if request.username == "admin" && request.password == "password" {
        // 生成JWT令牌
        let user_id = "user123";
        let roles = vec!["admin".to_string(), "trader".to_string()];
        let permissions = vec!["read".to_string(), "write".to_string(), "trade".to_string()];
        
        let access_token = match state.jwt_service.generate_access_token(
            user_id,
            &request.username,
            "admin@example.com",
            roles.clone(),
            permissions,
        ) {
            Ok(token) => token,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        let refresh_token = match state.jwt_service.generate_refresh_token(user_id) {
            Ok(token) => token,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        let response = LoginResponse {
            access_token,
            refresh_token,
            expires_in: state.config.auth.jwt_expiry,
            token_type: "Bearer".to_string(),
            user: UserInfo {
                id: user_id.to_string(),
                username: request.username,
                email: "admin@example.com".to_string(),
                roles,
            },
        };

        Ok(Json(ApiResponse::success(response)))
    } else {
        warn!("Invalid login attempt for user: {}", request.username);
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// 登出处理器
pub async fn logout(
    State(_state): State<AppState>,
    user_context: Option<axum::Extension<UserContext>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    if let Some(axum::Extension(user)) = user_context {
        info!("User logged out: {}", user.username);
        // 这里可以将令牌加入黑名单
    }

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "Successfully logged out"
    }))))
}

/// 刷新令牌处理器
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    // 验证刷新令牌
    let claims = match state.jwt_service.verify_token(&request.refresh_token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // 生成新的访问令牌
    let access_token = match state.jwt_service.generate_access_token(
        &claims.sub,
        &claims.username,
        &claims.email,
        claims.roles.clone(),
        claims.permissions.clone(),
    ) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 生成新的刷新令牌
    let new_refresh_token = match state.jwt_service.generate_refresh_token(&claims.sub) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let response = LoginResponse {
        access_token,
        refresh_token: new_refresh_token,
        expires_in: state.config.auth.jwt_expiry,
        token_type: "Bearer".to_string(),
        user: UserInfo {
            id: claims.sub,
            username: claims.username,
            email: claims.email,
            roles: claims.roles,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}