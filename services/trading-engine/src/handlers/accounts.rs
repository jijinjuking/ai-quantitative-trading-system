use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    services::AccountService,
    state::AppState,
};
use shared_models::AccountType;

#[derive(Debug, Deserialize)]
pub struct AccountQuery {
    pub account_type: Option<String>,
}

/// 获取账户信息
pub async fn get_account(
    State(state): State<AppState>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    let account_type = if let Some(type_str) = query.account_type {
        match type_str.parse::<AccountType>() {
            Ok(t) => Some(t),
            Err(_) => {
                let response = json!({
                    "success": false,
                    "error": "Invalid account type",
                    "message": "Invalid account type"
                });
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        None
    };

    match state.account_service.get_account(user_id, account_type).await {
        Ok(account) => {
            let response = json!({
                "success": true,
                "data": account
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to get account: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to get account"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取资金余额
pub async fn get_balance(
    State(state): State<AppState>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    let account_type = if let Some(type_str) = query.account_type {
        match type_str.parse::<AccountType>() {
            Ok(t) => Some(t),
            Err(_) => {
                let response = json!({
                    "success": false,
                    "error": "Invalid account type",
                    "message": "Invalid account type"
                });
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        None
    };

    match state.account_service.get_balance(user_id, account_type).await {
        Ok(balances) => {
            let response = json!({
                "success": true,
                "data": balances
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to get balance: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to get balance"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取保证金信息
pub async fn get_margin_info(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state.account_service.get_margin_info(user_id).await {
        Ok(margin_info) => {
            let response = json!({
                "success": true,
                "data": margin_info
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to get margin info: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to get margin info"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取盈亏统计
pub async fn get_pnl(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state.account_service.get_pnl_summary(user_id).await {
        Ok(pnl) => {
            let response = json!({
                "success": true,
                "data": pnl
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to get PnL: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to get PnL"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}