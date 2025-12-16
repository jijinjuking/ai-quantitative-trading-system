use anyhow::Result;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{CreateOrderRequest, Order, OrderStatus, TradingError, TradingResult},
    storage::OrderStore,
    services::{ExecutionService, RiskService},
};

/// 订单服务
#[derive(Clone)]
pub struct OrderService {
    order_store: Arc<OrderStore>,
    execution_service: Arc<ExecutionService>,
    risk_service: Arc<RiskService>,
}

impl OrderService {
    pub fn new(
        order_store: Arc<OrderStore>,
        execution_service: Arc<ExecutionService>,
        risk_service: Arc<RiskService>,
    ) -> Self {
        Self {
            order_store,
            execution_service,
            risk_service,
        }
    }

    /// 创建订单
    pub async fn create_order(
        &self,
        user_id: Uuid,
        request: CreateOrderRequest,
    ) -> TradingResult<Order> {
        // 1. 转换请求为订单
        let mut order = request.to_order(user_id)?;

        // 2. 风险检查
        self.risk_service.validate_order(&order).await?;

        // 3. 保存订单
        self.order_store.create_order(&order).await?;

        // 4. 提交执行
        match self.execution_service.submit_order(&order).await {
            Ok(_) => {
                tracing::info!("Order {} submitted for execution", order.id);
            }
            Err(e) => {
                tracing::error!("Failed to submit order {}: {}", order.id, e);
                // 标记订单为拒绝状态
                order.reject(&format!("Execution failed: {}", e))?;
                self.order_store.update_order(&order).await?;
                return Err(e);
            }
        }

        Ok(order)
    }

    /// 查询订单列表
    pub async fn list_orders(
        &self,
        user_id: Uuid,
        status: Option<String>,
        symbol: Option<String>,
        limit: u32,
        offset: u32,
    ) -> TradingResult<Vec<Order>> {
        let status_filter = if let Some(status_str) = status {
            Some(status_str.parse::<OrderStatus>().map_err(|e| {
                TradingError::InvalidOrder(format!("Invalid status: {}", e))
            })?)
        } else {
            None
        };

        self.order_store
            .list_orders(user_id, status_filter, symbol, limit, offset)
            .await
    }

    /// 查询单个订单
    pub async fn get_order(&self, user_id: Uuid, order_id: Uuid) -> TradingResult<Option<Order>> {
        self.order_store.get_order(user_id, order_id).await
    }

    /// 修改订单
    pub async fn update_order(
        &self,
        user_id: Uuid,
        order_id: Uuid,
        quantity: Option<Decimal>,
        price: Option<Decimal>,
    ) -> TradingResult<Order> {
        // 1. 获取订单
        let mut order = self
            .order_store
            .get_order(user_id, order_id)
            .await?
            .ok_or_else(|| TradingError::OrderNotFound(order_id))?;

        // 2. 检查订单状态
        if !order.status.can_modify() {
            return Err(TradingError::InvalidOrder(format!(
                "Cannot modify order in status: {}",
                order.status
            )));
        }

        // 3. 更新订单参数
        let mut modified = false;
        if let Some(new_quantity) = quantity {
            if new_quantity != order.quantity {
                order.quantity = new_quantity;
                order.remaining_quantity = new_quantity - order.filled_quantity;
                modified = true;
            }
        }

        if let Some(new_price) = price {
            if order.price != Some(new_price) {
                order.price = Some(new_price);
                modified = true;
            }
        }

        if !modified {
            return Ok(order);
        }

        // 4. 验证修改后的订单
        order.validate()?;

        // 5. 风险检查
        self.risk_service.validate_order(&order).await?;

        // 6. 更新时间戳
        order.updated_at = chrono::Utc::now();

        // 7. 保存订单
        self.order_store.update_order(&order).await?;

        // 8. 通知执行服务
        self.execution_service.update_order(&order).await?;

        Ok(order)
    }

    /// 取消订单
    pub async fn cancel_order(&self, user_id: Uuid, order_id: Uuid) -> TradingResult<Order> {
        // 1. 获取订单
        let mut order = self
            .order_store
            .get_order(user_id, order_id)
            .await?
            .ok_or_else(|| TradingError::OrderNotFound(order_id))?;

        // 2. 取消订单
        order.cancel()?;

        // 3. 保存订单
        self.order_store.update_order(&order).await?;

        // 4. 通知执行服务
        self.execution_service.cancel_order(&order).await?;

        Ok(order)
    }

    /// 处理订单成交
    pub async fn handle_order_fill(
        &self,
        order_id: Uuid,
        fill_quantity: Decimal,
        fill_price: Decimal,
        fee: Decimal,
    ) -> TradingResult<()> {
        // 1. 获取订单
        let mut order = self
            .order_store
            .get_order_by_id(order_id)
            .await?
            .ok_or_else(|| TradingError::OrderNotFound(order_id))?;

        // 2. 更新成交信息
        order.update_fill(fill_quantity, fill_price, fee)?;

        // 3. 保存订单
        self.order_store.update_order(&order).await?;

        // 4. 如果订单完全成交，通知相关服务
        if order.status == OrderStatus::Filled {
            tracing::info!("Order {} fully filled", order_id);
        }

        Ok(())
    }

    /// 获取活跃订单
    pub async fn get_active_orders(&self, user_id: Uuid) -> TradingResult<Vec<Order>> {
        self.order_store.get_active_orders(user_id).await
    }

    /// 取消所有订单
    pub async fn cancel_all_orders(&self, user_id: Uuid, symbol: Option<String>) -> TradingResult<Vec<Order>> {
        let active_orders = if let Some(symbol) = symbol {
            self.order_store
                .list_orders(user_id, Some(OrderStatus::Pending), Some(symbol), 1000, 0)
                .await?
        } else {
            self.get_active_orders(user_id).await?
        };

        let mut cancelled_orders = Vec::new();
        for order_id in active_orders.iter().map(|o| o.id) {
            match self.cancel_order(user_id, order_id).await {
                Ok(order) => cancelled_orders.push(order),
                Err(e) => {
                    tracing::error!("Failed to cancel order {}: {}", order_id, e);
                }
            }
        }

        Ok(cancelled_orders)
    }

    /// 检查订单过期
    pub async fn check_expired_orders(&self) -> TradingResult<()> {
        let expired_orders = self.order_store.get_expired_orders().await?;
        
        for mut order in expired_orders {
            if let Err(e) = order.expire() {
                tracing::error!("Failed to expire order {}: {}", order.id, e);
                continue;
            }

            if let Err(e) = self.order_store.update_order(&order).await {
                tracing::error!("Failed to save expired order {}: {}", order.id, e);
            } else {
                tracing::info!("Order {} expired", order.id);
            }
        }

        Ok(())
    }
}