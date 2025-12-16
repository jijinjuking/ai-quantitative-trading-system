use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as RequestJson,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    models::{Position, PositionSummary},
    services::PositionService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListPositionsQuery {
    pub status: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClosePositionRequest {
    pub symbol: String,
    pub size: Option<rust_decimal::Decimal>,
    pub price: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CloseAllPositionsRequest {
    pub symbols: Option<Vec<String>>,
}

/// 查询仓位列表
pub async fn list_positions(
    State(state): State<AppState>,
    Query(query): Query<ListPositionsQuery>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state
        .position_service
        .list_positions(user_id, query.status, query.symbol)
        .await
    {
        Ok(positions) => {
            let summaries: Vec<PositionSummary> = positions.iter().map(|p| p.into()).collect();
            let response = json!({
                "success": true,
                "data": summaries,
                "count": positions.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to list positions: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to list positions"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 查询单个仓位
pub async fn get_position(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state.position_service.get_position(user_id, &symbol).await {
        Ok(Some(position)) => {
            let summary: PositionSummary = (&position).into();
            let response = json!({
                "success": true,
                "data": summary
            });
            Ok(Json(response))
        }
        Ok(None) => {
            let response = json!({
                "success": false,
                "error": "Position not found",
                "message": "Position not found"
            });
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to get position: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to get position"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 平仓
pub async fn close_position(
    State(state): State<AppState>,
    RequestJson(request): RequestJson<ClosePositionRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state
        .position_service
        .close_position(user_id, &request.symbol, request.size, request.price)
        .await
    {
        Ok(result) => {
            let response = json!({
                "success": true,
                "data": result,
                "message": "Position closed successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to close position: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to close position"
            });
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 全部平仓
pub async fn close_all_positions(
    State(state): State<AppState>,
    RequestJson(request): RequestJson<CloseAllPositionsRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state
        .position_service
        .close_all_positions(user_id, request.symbols)
        .await
    {
        Ok(results) => {
            let response = json!({
                "success": true,
                "data": results,
                "message": "All positions closed successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to close all positions: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to close all positions"
            });
            Err(StatusCode::BAD_REQUEST)
        }
    }
}