use anyhow::Result;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Order, OrderStatus, OrderType, Side, Symbol, TimeInForce, TradingError, TradingResult};

/// 订单存储
#[derive(Clone)]
pub struct OrderStore {
    pool: Arc<PgPool>,
}

impl OrderStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 创建订单
    pub async fn create_order(&self, order: &Order) -> TradingResult<()> {
        let query = r#"
            INSERT INTO orders (
                id, user_id, symbol, order_type, side, quantity, price, stop_price,
                status, time_in_force, filled_quantity, remaining_quantity,
                average_price, fee, fee_currency, created_at, updated_at,
                expires_at, client_order_id, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20
            )
        "#;

        sqlx::query(query)
            .bind(order.id)
            .bind(order.user_id)
            .bind(order.symbol.to_string())
            .bind(order.order_type.to_string())
            .bind(order.side.to_string())
            .bind(order.quantity)
            .bind(order.price)
            .bind(order.stop_price)
            .bind(order.status.to_string())
            .bind(order.time_in_force.to_string())
            .bind(order.filled_quantity)
            .bind(order.remaining_quantity)
            .bind(order.average_price)
            .bind(order.fee)
            .bind(&order.fee_currency)
            .bind(order.created_at)
            .bind(order.updated_at)
            .bind(order.expires_at)
            .bind(&order.client_order_id)
            .bind(serde_json::to_value(&order.metadata).unwrap())
            .execute(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 更新订单
    pub async fn update_order(&self, order: &Order) -> TradingResult<()> {
        let query = r#"
            UPDATE orders SET
                status = $2, filled_quantity = $3, remaining_quantity = $4,
                average_price = $5, fee = $6, updated_at = $7, metadata = $8
            WHERE id = $1
        "#;

        let result = sqlx::query(query)
            .bind(order.id)
            .bind(order.status.to_string())
            .bind(order.filled_quantity)
            .bind(order.remaining_quantity)
            .bind(order.average_price)
            .bind(order.fee)
            .bind(order.updated_at)
            .bind(serde_json::to_value(&order.metadata).unwrap())
            .execute(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TradingError::OrderNotFound(order.id));
        }

        Ok(())
    }

    /// 查询订单
    pub async fn get_order(&self, user_id: Uuid, order_id: Uuid) -> TradingResult<Option<Order>> {
        let query = r#"
            SELECT * FROM orders WHERE id = $1 AND user_id = $2
        "#;

        let row = sqlx::query(query)
            .bind(order_id)
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_order(row)?))
        } else {
            Ok(None)
        }
    }

    /// 根据ID查询订单（不检查用户）
    pub async fn get_order_by_id(&self, order_id: Uuid) -> TradingResult<Option<Order>> {
        let query = r#"
            SELECT * FROM orders WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(order_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_order(row)?))
        } else {
            Ok(None)
        }
    }

    /// 查询订单列表
    pub async fn list_orders(
        &self,
        user_id: Uuid,
        status: Option<OrderStatus>,
        symbol: Option<String>,
        limit: u32,
        offset: u32,
    ) -> TradingResult<Vec<Order>> {
        let mut query = "SELECT * FROM orders WHERE user_id = $1".to_string();
        let mut param_count = 1;

        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        if symbol.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND symbol = ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        let mut sql_query = sqlx::query(&query).bind(user_id);

        if let Some(status) = status {
            sql_query = sql_query.bind(status.to_string());
        }

        if let Some(symbol) = symbol {
            sql_query = sql_query.bind(symbol);
        }

        sql_query = sql_query.bind(limit as i64).bind(offset as i64);

        let rows = sql_query
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(self.row_to_order(row)?);
        }

        Ok(orders)
    }

    /// 获取活跃订单
    pub async fn get_active_orders(&self, user_id: Uuid) -> TradingResult<Vec<Order>> {
        let query = r#"
            SELECT * FROM orders 
            WHERE user_id = $1 AND status IN ('PENDING', 'PARTIALLY_FILLED')
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(self.row_to_order(row)?);
        }

        Ok(orders)
    }

    /// 获取过期订单
    pub async fn get_expired_orders(&self) -> TradingResult<Vec<Order>> {
        let query = r#"
            SELECT * FROM orders 
            WHERE expires_at IS NOT NULL 
            AND expires_at < $1 
            AND status IN ('PENDING', 'PARTIALLY_FILLED')
        "#;

        let rows = sqlx::query(query)
            .bind(Utc::now())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(self.row_to_order(row)?);
        }

        Ok(orders)
    }

    /// 删除订单
    pub async fn delete_order(&self, user_id: Uuid, order_id: Uuid) -> TradingResult<()> {
        let query = r#"
            DELETE FROM orders WHERE id = $1 AND user_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(order_id)
            .bind(user_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| TradingError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TradingError::OrderNotFound(order_id));
        }

        Ok(())
    }

    /// 将数据库行转换为订单对象
    fn row_to_order(&self, row: sqlx::postgres::PgRow) -> TradingResult<Order> {
        let symbol_str: String = row.get("symbol");
        let symbol = Symbol::from_string(&symbol_str)
            .ok_or_else(|| TradingError::InvalidOrder(format!("Invalid symbol: {}", symbol_str)))?;

        let order_type_str: String = row.get("order_type");
        let order_type = order_type_str
            .parse::<OrderType>()
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid order type: {}", e)))?;

        let side_str: String = row.get("side");
        let side = side_str
            .parse::<Side>()
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid side: {}", e)))?;

        let status_str: String = row.get("status");
        let status = status_str
            .parse::<OrderStatus>()
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid status: {}", e)))?;

        let tif_str: String = row.get("time_in_force");
        let time_in_force = match tif_str.as_str() {
            "GTC" => TimeInForce::GTC,
            "IOC" => TimeInForce::IOC,
            "FOK" => TimeInForce::FOK,
            "GTD" => TimeInForce::GTD,
            _ => return Err(TradingError::InvalidOrder(format!("Invalid time in force: {}", tif_str))),
        };

        let metadata_json: serde_json::Value = row.get("metadata");
        let metadata = serde_json::from_value(metadata_json)
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid metadata: {}", e)))?;

        Ok(Order {
            id: row.get("id"),
            user_id: row.get("user_id"),
            symbol,
            order_type,
            side,
            quantity: row.get("quantity"),
            price: row.get("price"),
            stop_price: row.get("stop_price"),
            status,
            time_in_force,
            filled_quantity: row.get("filled_quantity"),
            remaining_quantity: row.get("remaining_quantity"),
            average_price: row.get("average_price"),
            fee: row.get("fee"),
            fee_currency: row.get("fee_currency"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            expires_at: row.get("expires_at"),
            client_order_id: row.get("client_order_id"),
            metadata,
        })
    }
}