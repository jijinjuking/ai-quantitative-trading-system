use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    config::TradingEngineConfig,
    models::{Order, Position, TradingError, TradingResult},
};

/// 风险管理服务
#[derive(Clone)]
pub struct RiskService {
    config: TradingEngineConfig,
    // 风险限制配置
    max_position_size: HashMap<String, Decimal>,
    max_order_value: Decimal,
    max_daily_loss: Decimal,
    maintenance_margin_rate: Decimal,
}

impl RiskService {
    pub fn new(config: TradingEngineConfig) -> Self {
        let mut max_position_size = HashMap::new();
        max_position_size.insert("BTCUSDT".to_string(), Decimal::from(10));
        max_position_size.insert("ETHUSDT".to_string(), Decimal::from(100));
        max_position_size.insert("ADAUSDT".to_string(), Decimal::from(10000));

        Self {
            config,
            max_position_size,
            max_order_value: Decimal::from(100000), // 10万USDT
            max_daily_loss: Decimal::from(10000),   // 1万USDT
            maintenance_margin_rate: Decimal::new(5, 2), // 5%
        }
    }

    /// 验证订单风险
    pub async fn validate_order(&self, order: &Order) -> TradingResult<()> {
        // 1. 检查订单数量限制
        self.check_position_size_limit(order)?;

        // 2. 检查订单价值限制
        self.check_order_value_limit(order)?;

        // 3. 检查价格合理性
        self.check_price_reasonableness(order).await?;

        // 4. 检查用户风险限制
        self.check_user_risk_limits(order).await?;

        Ok(())
    }

    /// 验证仓位平仓
    pub async fn validate_position_close(
        &self,
        position: &Position,
        close_size: Decimal,
        close_price: Decimal,
    ) -> TradingResult<()> {
        // 1. 检查平仓数量
        if close_size <= Decimal::ZERO || close_size > position.size {
            return Err(TradingError::InvalidOrder(
                "Invalid close size".to_string(),
            ));
        }

        // 2. 检查平仓价格合理性
        if close_price <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Invalid close price".to_string(),
            ));
        }

        // 3. 检查价格偏离度
        let price_deviation = (close_price - position.mark_price).abs() / position.mark_price;
        if price_deviation > Decimal::new(1, 1) {
            // 超过10%偏离
            return Err(TradingError::RiskViolation(
                "Close price deviates too much from mark price".to_string(),
            ));
        }

        Ok(())
    }

    /// 检查仓位大小限制
    fn check_position_size_limit(&self, order: &Order) -> TradingResult<()> {
        let symbol = order.symbol.to_string();
        if let Some(&max_size) = self.max_position_size.get(&symbol) {
            if order.quantity > max_size {
                return Err(TradingError::RiskViolation(format!(
                    "Order quantity {} exceeds maximum position size {} for {}",
                    order.quantity, max_size, symbol
                )));
            }
        }
        Ok(())
    }

    /// 检查订单价值限制
    fn check_order_value_limit(&self, order: &Order) -> TradingResult<()> {
        if let Some(order_value) = order.calculate_value() {
            if order_value > self.max_order_value {
                return Err(TradingError::RiskViolation(format!(
                    "Order value {} exceeds maximum order value {}",
                    order_value, self.max_order_value
                )));
            }
        }
        Ok(())
    }

    /// 检查价格合理性
    async fn check_price_reasonableness(&self, order: &Order) -> TradingResult<()> {
        if let Some(order_price) = order.price {
            // TODO: 从市场数据服务获取当前市价
            let market_price = self.get_market_price(&order.symbol.to_string()).await?;
            
            // 检查价格偏离度（不能超过20%）
            let price_deviation = (order_price - market_price).abs() / market_price;
            if price_deviation > Decimal::new(2, 1) {
                // 超过20%偏离
                return Err(TradingError::RiskViolation(format!(
                    "Order price {} deviates too much from market price {}",
                    order_price, market_price
                )));
            }
        }
        Ok(())
    }

    /// 检查用户风险限制
    async fn check_user_risk_limits(&self, order: &Order) -> TradingResult<()> {
        // TODO: 实现用户级别的风险检查
        // 1. 检查用户当日亏损
        // 2. 检查用户总仓位
        // 3. 检查用户保证金使用率
        // 4. 检查用户交易频率

        Ok(())
    }

    /// 获取市场价格（临时实现）
    async fn get_market_price(&self, symbol: &str) -> TradingResult<Decimal> {
        // TODO: 从市场数据服务获取实时价格
        match symbol {
            "BTCUSDT" => Ok(Decimal::from(45000)),
            "ETHUSDT" => Ok(Decimal::from(3000)),
            "ADAUSDT" => Ok(Decimal::new(5, 1)), // 0.5
            _ => Ok(Decimal::from(100)), // 默认价格
        }
    }

    /// 计算保证金要求
    pub fn calculate_margin_requirement(
        &self,
        symbol: &str,
        size: Decimal,
        price: Decimal,
        leverage: Decimal,
    ) -> Decimal {
        let position_value = size * price;
        position_value / leverage
    }

    /// 检查保证金充足性
    pub fn check_margin_sufficiency(
        &self,
        available_margin: Decimal,
        required_margin: Decimal,
    ) -> TradingResult<()> {
        if available_margin < required_margin {
            return Err(TradingError::InsufficientMargin {
                required: required_margin,
                available: available_margin,
            });
        }
        Ok(())
    }

    /// 计算强平价格
    pub fn calculate_liquidation_price(
        &self,
        entry_price: Decimal,
        size: Decimal,
        margin: Decimal,
        is_long: bool,
    ) -> Option<Decimal> {
        let position_value = entry_price * size;
        let maintenance_margin = position_value * self.maintenance_margin_rate;
        let available_margin = margin - maintenance_margin;

        if available_margin <= Decimal::ZERO {
            return Some(entry_price);
        }

        let liquidation_price = if is_long {
            entry_price - (available_margin / size)
        } else {
            entry_price + (available_margin / size)
        };

        if liquidation_price > Decimal::ZERO {
            Some(liquidation_price)
        } else {
            None
        }
    }

    /// 检查是否需要强平
    pub fn should_liquidate(&self, position: &Position) -> bool {
        position.should_liquidate(self.maintenance_margin_rate)
    }

    /// 获取风险指标
    pub fn get_risk_metrics(&self, positions: &[Position]) -> RiskMetrics {
        let total_margin: Decimal = positions.iter().map(|p| p.margin).sum();
        let total_unrealized_pnl: Decimal = positions.iter().map(|p| p.unrealized_pnl).sum();
        let total_position_value: Decimal = positions.iter().map(|p| p.get_position_value()).sum();

        let margin_usage_rate = if total_position_value > Decimal::ZERO {
            total_margin / total_position_value
        } else {
            Decimal::ZERO
        };

        RiskMetrics {
            total_margin,
            total_unrealized_pnl,
            total_position_value,
            margin_usage_rate,
            positions_at_risk: positions
                .iter()
                .filter(|p| self.should_liquidate(p))
                .count(),
        }
    }
}

/// 风险指标
#[derive(Debug, serde::Serialize)]
pub struct RiskMetrics {
    pub total_margin: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub total_position_value: Decimal,
    pub margin_usage_rate: Decimal,
    pub positions_at_risk: usize,
}