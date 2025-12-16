use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::state::AppState;

pub mod accounts;
pub mod health;
pub mod orders;
pub mod positions;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 健康检查
        .route("/health", get(health::health_check))
        // 订单管理
        .route("/api/v1/orders", post(orders::create_order))
        .route("/api/v1/orders", get(orders::list_orders))
        .route("/api/v1/orders/:id", get(orders::get_order))
        .route("/api/v1/orders/:id", put(orders::update_order))
        .route("/api/v1/orders/:id", delete(orders::cancel_order))
        .route("/api/v1/orders/batch", post(orders::batch_orders))
        // 仓位管理
        .route("/api/v1/positions", get(positions::list_positions))
        .route("/api/v1/positions/:symbol", get(positions::get_position))
        .route("/api/v1/positions/close", post(positions::close_position))
        .route(
            "/api/v1/positions/close-all",
            post(positions::close_all_positions),
        )
        // 账户管理
        .route("/api/v1/account", get(accounts::get_account))
        .route("/api/v1/account/balance", get(accounts::get_balance))
        .route("/api/v1/account/margin", get(accounts::get_margin_info))
        .route("/api/v1/account/pnl", get(accounts::get_pnl))
        // WebSocket
        .route(
            "/ws/orders",
            get(crate::websocket::orders::orders_websocket),
        )
        .route(
            "/ws/positions",
            get(crate::websocket::positions::positions_websocket),
        )
        .route(
            "/ws/account",
            get(crate::websocket::account::account_websocket),
        )
        // 指标
        .route("/metrics", get(crate::handlers::health::metrics))
}
