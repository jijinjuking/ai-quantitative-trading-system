use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as RequestJson,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::{CreateOrderRequest, Order, OrderStatus},
    services::OrderService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub status: Option<String>,
    pub symbol: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderRequest {
    pub quantity: Option<rust_decimal::Decimal>,
    pub price: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct BatchOrderRequest {
    pub orders: Vec<CreateOrderRequest>,
}

/// 创建订单
pub async fn create_order(
    State(state): State<AppState>,
    RequestJson(request): RequestJson<CreateOrderRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state.order_service.create_order(user_id, request).await {
        Ok(order) => {
            let response = json!({
                "success": true,
                "data": order,
                "message": "Order created successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to create order: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to create order"
            });
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 查询订单列表
pub async fn list_orders(
    State(state): State<AppState>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    match state
        .order_service
        .list_orders(user_id, query.status, query.symbol, limit, offset)
        .await
    {
        Ok(orders) => {
            let response = json!({
                "success": true,
                "data": orders,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "total": orders.len()
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to list orders: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to list orders"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 查询单个订单
pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state.order_service.get_order(user_id, order_id).await {
        Ok(Some(order)) => {
            let response = json!({
                "success": true,
                "data": order
            });
            Ok(Json(response))
        }
        Ok(None) => {
            let response = json!({
                "success": false,
                "error": "Order not found",
                "message": "Order not found"
            });
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to get order: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to get order"
            });
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 修改订单
pub async fn update_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    RequestJson(request): RequestJson<UpdateOrderRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state
        .order_service
        .update_order(user_id, order_id, request.quantity, request.price)
        .await
    {
        Ok(order) => {
            let response = json!({
                "success": true,
                "data": order,
                "message": "Order updated successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to update order: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to update order"
            });
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 取消订单
pub async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    match state.order_service.cancel_order(user_id, order_id).await {
        Ok(order) => {
            let response = json!({
                "success": true,
                "data": order,
                "message": "Order cancelled successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to cancel order: {}", e);
            let response = json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to cancel order"
            });
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 批量操作订单
pub async fn batch_orders(
    State(state): State<AppState>,
    RequestJson(request): RequestJson<BatchOrderRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    let mut results = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for order_request in request.orders {
        match state.order_service.create_order(user_id, order_request).await {
            Ok(order) => {
                results.push(json!({
                    "success": true,
                    "data": order
                }));
                success_count += 1;
            }
            Err(e) => {
                results.push(json!({
                    "success": false,
                    "error": e.to_string()
                }));
                error_count += 1;
            }
        }
    }

    let response = json!({
        "success": error_count == 0,
        "data": results,
        "summary": {
            "total": results.len(),
            "success": success_count,
            "errors": error_count
        }
    });

    if error_count > 0 {
        Err(StatusCode::PARTIAL_CONTENT)
    } else {
        Ok(Json(response))
    }
}