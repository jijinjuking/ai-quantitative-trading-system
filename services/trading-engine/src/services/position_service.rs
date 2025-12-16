use anyhow::Result;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{Position, PositionStatus, PositionSide, Side, Symbol, TradingError, TradingResult},
    storage::PositionStore,
    services::{ExecutionService, RiskService},
};

/// 仓位服务
#[derive(Clone)]
pub struct PositionService {
    position_store: Arc<PositionStore>,
    execution_service: Arc<ExecutionService>,
    risk_service: Arc<RiskService>,
}

#[derive(Debug, serde::Serialize)]
pub struct ClosePositionResult {
    pub position_id: Uuid,
    pub symbol: String,
    pub closed_size: Decimal,
    pub close_price: Decimal,
    pub realized_pnl: Decimal,
    pub remaining_size: Decimal,
}

impl PositionService {
    pub fn new(
        position_store: Arc<PositionStore>,
        execution_service: Arc<ExecutionService>,
        risk_service: Arc<RiskService>,
    ) -> Self {
        Self {
            position_store,
            execution_service,
            risk_service,
        }
    }

    /// 查询仓位列表
    pub async fn list_positions(
        &self,
        user_id: Uuid,
        status: Option<String>,
        symbol: Option<String>,
    ) -> TradingResult<Vec<Position>> {
        let status_filter = if let Some(status_str) = status {
            Some(status_str.parse::<PositionStatus>().map_err(|e| {
                TradingError::InvalidOrder(format!("Invalid status: {}", e))
            })?)
        } else {
            None
        };

        self.position_store
            .list_positions(user_id, status_filter, symbol)
            .await
    }

    /// 查询单个仓位
    pub async fn get_position(
        &self,
        user_id: Uuid,
        symbol: &str,
    ) -> TradingResult<Option<Position>> {
        self.position_store.get_position(user_id, symbol).await
    }

    /// 创建或更新仓位
    pub async fn update_position(
        &self,
        user_id: Uuid,
        symbol: Symbol,
        side: Side,
        size: Decimal,
        price: Decimal,
        leverage: Decimal,
        margin: Decimal,
    ) -> TradingResult<Position> {
        let symbol_str = symbol.to_string();
        
        // 检查是否已有仓位
        if let Some(mut existing_position) = self.get_position(user_id, &symbol_str).await? {
            let position_side = PositionSide::from_side(side);
            
            if existing_position.side == position_side {
                // 同方向，增加仓位
                existing_position.increase_position(size, price, margin)?;
                self.position_store.update_position(&existing_position).await?;
                Ok(existing_position)
            } else {
                // 反方向，可能是平仓或反向开仓
                if size <= existing_position.size {
                    // 部分或完全平仓
                    let pnl = existing_position.partial_close(size, price)?;
                    self.position_store.update_position(&existing_position).await?;
                    
                    tracing::info!(
                        "Position partially closed: {} {} {}, PnL: {}",
                        symbol_str, size, side, pnl
                    );
                    
                    Ok(existing_position)
                } else {
                    // 完全平仓并反向开仓
                    let close_size = existing_position.size;
                    let pnl = existing_position.close(price)?;
                    self.position_store.update_position(&existing_position).await?;
                    
                    // 创建新的反向仓位
                    let new_size = size - close_size;
                    let new_position = Position::new(
                        user_id,
                        symbol,
                        position_side,
                        new_size,
                        price,
                        leverage,
                        margin,
                    )?;
                    
                    self.position_store.create_position(&new_position).await?;
                    
                    tracing::info!(
                        "Position closed and reversed: {} {} -> {} {}, PnL: {}",
                        symbol_str, close_size, new_size, side, pnl
                    );
                    
                    Ok(new_position)
                }
            }
        } else {
            // 创建新仓位
            let position_side = PositionSide::from_side(side);
            let position = Position::new(user_id, symbol, position_side, size, price, leverage, margin)?;
            
            self.position_store.create_position(&position).await?;
            
            tracing::info!(
                "New position created: {} {} {}",
                symbol_str, size, side
            );
            
            Ok(position)
        }
    }

    /// 平仓
    pub async fn close_position(
        &self,
        user_id: Uuid,
        symbol: &str,
        size: Option<Decimal>,
        price: Option<Decimal>,
    ) -> TradingResult<ClosePositionResult> {
        // 1. 获取仓位
        let mut position = self
            .get_position(user_id, symbol)
            .await?
            .ok_or_else(|| TradingError::PositionNotFound(symbol.to_string()))?;

        if position.status == PositionStatus::Closed {
            return Err(TradingError::InvalidOrder(
                "Position is already closed".to_string(),
            ));
        }

        // 2. 确定平仓数量
        let close_size = size.unwrap_or(position.size);
        if close_size <= Decimal::ZERO || close_size > position.size {
            return Err(TradingError::InvalidOrder(
                "Invalid close size".to_string(),
            ));
        }

        // 3. 确定平仓价格（如果没有指定，使用当前市价）
        let close_price = if let Some(p) = price {
            p
        } else {
            // TODO: 从市场数据服务获取当前价格
            position.mark_price
        };

        // 4. 风险检查
        self.risk_service.validate_position_close(&position, close_size, close_price).await?;

        // 5. 执行平仓
        let close_side = position.side.to_close_side();
        let order_result = self
            .execution_service
            .create_market_order(user_id, symbol.to_string(), close_side, close_size)
            .await?;

        // 6. 更新仓位
        let pnl = position.partial_close(close_size, close_price)?;
        self.position_store.update_position(&position).await?;

        let result = ClosePositionResult {
            position_id: position.id,
            symbol: symbol.to_string(),
            closed_size: close_size,
            close_price,
            realized_pnl: pnl,
            remaining_size: position.size,
        };

        tracing::info!(
            "Position closed: {} {} {}, PnL: {}",
            symbol, close_size, close_side, pnl
        );

        Ok(result)
    }

    /// 全部平仓
    pub async fn close_all_positions(
        &self,
        user_id: Uuid,
        symbols: Option<Vec<String>>,
    ) -> TradingResult<Vec<ClosePositionResult>> {
        let positions = if let Some(symbol_list) = symbols {
            let mut filtered_positions = Vec::new();
            for symbol in symbol_list {
                if let Some(position) = self.get_position(user_id, &symbol).await? {
                    if position.status != PositionStatus::Closed {
                        filtered_positions.push(position);
                    }
                }
            }
            filtered_positions
        } else {
            self.list_positions(user_id, Some("OPEN".to_string()), None).await?
        };

        let mut results = Vec::new();
        for position in positions {
            match self
                .close_position(user_id, &position.symbol.to_string(), None, None)
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!(
                        "Failed to close position {}: {}",
                        position.symbol.to_string(),
                        e
                    );
                }
            }
        }

        Ok(results)
    }

    /// 更新仓位标记价格
    pub async fn update_mark_prices(&self, prices: std::collections::HashMap<String, Decimal>) -> TradingResult<()> {
        for (symbol, price) in prices {
            if let Err(e) = self.update_symbol_mark_price(&symbol, price).await {
                tracing::error!("Failed to update mark price for {}: {}", symbol, e);
            }
        }
        Ok(())
    }

    /// 更新单个交易对的标记价格
    async fn update_symbol_mark_price(&self, symbol: &str, mark_price: Decimal) -> TradingResult<()> {
        let positions = self.position_store.get_positions_by_symbol(symbol).await?;
        
        for mut position in positions {
            if position.status == PositionStatus::Open {
                position.update_mark_price(mark_price)?;
                self.position_store.update_position(&position).await?;
            }
        }
        
        Ok(())
    }

    /// 检查需要强平的仓位
    pub async fn check_liquidation(&self, maintenance_margin_rate: Decimal) -> TradingResult<Vec<Position>> {
        let all_positions = self.position_store.get_all_active_positions().await?;
        let mut liquidation_positions = Vec::new();

        for position in all_positions {
            if position.should_liquidate(maintenance_margin_rate) {
                liquidation_positions.push(position);
            }
        }

        Ok(liquidation_positions)
    }

    /// 执行强平
    pub async fn liquidate_position(&self, position_id: Uuid) -> TradingResult<ClosePositionResult> {
        let position = self
            .position_store
            .get_position_by_id(position_id)
            .await?
            .ok_or_else(|| TradingError::PositionNotFound(position_id.to_string()))?;

        if position.status != PositionStatus::Open {
            return Err(TradingError::InvalidOrder(
                "Position is not open for liquidation".to_string(),
            ));
        }

        // 使用当前标记价格强制平仓
        self.close_position(
            position.user_id,
            &position.symbol.to_string(),
            None,
            Some(position.mark_price),
        )
        .await
    }

    /// 计算用户总盈亏
    pub async fn calculate_total_pnl(&self, user_id: Uuid) -> TradingResult<Decimal> {
        let positions = self.list_positions(user_id, None, None).await?;
        let total_pnl = positions.iter().map(|p| p.get_total_pnl()).sum();
        Ok(total_pnl)
    }

    /// 计算用户总保证金
    pub async fn calculate_total_margin(&self, user_id: Uuid) -> TradingResult<Decimal> {
        let positions = self.list_positions(user_id, Some("OPEN".to_string()), None).await?;
        let total_margin = positions.iter().map(|p| p.margin).sum();
        Ok(total_margin)
    }
}