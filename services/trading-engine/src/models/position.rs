use super::{Amount, Id, Price, Quantity, Side, Symbol, Timestamp, TradingError, TradingResult};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 仓位方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    pub fn opposite(&self) -> Self {
        match self {
            PositionSide::Long => PositionSide::Short,
            PositionSide::Short => PositionSide::Long,
        }
    }

    pub fn is_long(&self) -> bool {
        matches!(self, PositionSide::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(self, PositionSide::Short)
    }

    /// 从交易方向推导仓位方向
    pub fn from_side(side: Side) -> Self {
        match side {
            Side::Buy => PositionSide::Long,
            Side::Sell => PositionSide::Short,
        }
    }

    /// 转换为交易方向（平仓时使用）
    pub fn to_close_side(&self) -> Side {
        match self {
            PositionSide::Long => Side::Sell,
            PositionSide::Short => Side::Buy,
        }
    }
}

impl std::fmt::Display for PositionSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionSide::Long => write!(f, "LONG"),
            PositionSide::Short => write!(f, "SHORT"),
        }
    }
}

impl std::str::FromStr for PositionSide {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LONG" | "L" => Ok(PositionSide::Long),
            "SHORT" | "S" => Ok(PositionSide::Short),
            _ => Err(anyhow::anyhow!("Invalid position side: {}", s)),
        }
    }
}

/// 仓位状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closing,
    Closed,
}

impl PositionStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, PositionStatus::Open | PositionStatus::Closing)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self, PositionStatus::Closed)
    }
}

impl std::str::FromStr for PositionStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "OPEN" => Ok(PositionStatus::Open),
            "CLOSING" => Ok(PositionStatus::Closing),
            "CLOSED" => Ok(PositionStatus::Closed),
            _ => Err(anyhow::anyhow!("Invalid position status: {}", s)),
        }
    }
}

impl std::fmt::Display for PositionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionStatus::Open => write!(f, "OPEN"),
            PositionStatus::Closing => write!(f, "CLOSING"),
            PositionStatus::Closed => write!(f, "CLOSED"),
        }
    }
}

/// 仓位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Id,
    pub user_id: Id,
    pub symbol: Symbol,
    pub side: PositionSide,
    pub size: Quantity,
    pub entry_price: Price,
    pub mark_price: Price,
    pub liquidation_price: Option<Price>,
    pub unrealized_pnl: Amount,
    pub realized_pnl: Amount,
    pub margin: Amount,
    pub margin_ratio: Decimal,
    pub leverage: Decimal,
    pub status: PositionStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub closed_at: Option<Timestamp>,
}

impl Position {
    /// 创建新仓位
    pub fn new(
        user_id: Id,
        symbol: Symbol,
        side: PositionSide,
        size: Quantity,
        entry_price: Price,
        leverage: Decimal,
        margin: Amount,
    ) -> TradingResult<Self> {
        if size <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Position size must be positive".to_string(),
            ));
        }

        if entry_price <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Entry price must be positive".to_string(),
            ));
        }

        if leverage <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Leverage must be positive".to_string(),
            ));
        }

        if margin <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Margin must be positive".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            symbol,
            side,
            size,
            entry_price,
            mark_price: entry_price,
            liquidation_price: None,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            margin,
            margin_ratio: Decimal::ZERO,
            leverage,
            status: PositionStatus::Open,
            created_at: now,
            updated_at: now,
            closed_at: None,
        })
    }

    /// 更新标记价格和未实现盈亏
    pub fn update_mark_price(&mut self, mark_price: Price) -> TradingResult<()> {
        if mark_price <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Mark price must be positive".to_string(),
            ));
        }

        self.mark_price = mark_price;
        self.unrealized_pnl = self.calculate_unrealized_pnl();
        self.margin_ratio = self.calculate_margin_ratio();
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 计算未实现盈亏
    pub fn calculate_unrealized_pnl(&self) -> Amount {
        let price_diff = match self.side {
            PositionSide::Long => self.mark_price - self.entry_price,
            PositionSide::Short => self.entry_price - self.mark_price,
        };
        price_diff * self.size
    }

    /// 计算保证金比率
    pub fn calculate_margin_ratio(&self) -> Decimal {
        let position_value = self.mark_price * self.size;
        if position_value > Decimal::ZERO {
            (self.margin + self.unrealized_pnl) / position_value
        } else {
            Decimal::ZERO
        }
    }

    /// 计算强平价格
    pub fn calculate_liquidation_price(&self, maintenance_margin_rate: Decimal) -> Option<Price> {
        if maintenance_margin_rate <= Decimal::ZERO || maintenance_margin_rate >= Decimal::ONE {
            return None;
        }

        let position_value = self.entry_price * self.size;
        let maintenance_margin = position_value * maintenance_margin_rate;
        let available_margin = self.margin - maintenance_margin;

        if available_margin <= Decimal::ZERO {
            return Some(self.entry_price);
        }

        let liquidation_price = match self.side {
            PositionSide::Long => self.entry_price - (available_margin / self.size),
            PositionSide::Short => self.entry_price + (available_margin / self.size),
        };

        if liquidation_price > Decimal::ZERO {
            Some(liquidation_price)
        } else {
            None
        }
    }

    /// 更新强平价格
    pub fn update_liquidation_price(&mut self, maintenance_margin_rate: Decimal) {
        self.liquidation_price = self.calculate_liquidation_price(maintenance_margin_rate);
        self.updated_at = Utc::now();
    }

    /// 检查是否需要强平
    pub fn should_liquidate(&self, maintenance_margin_rate: Decimal) -> bool {
        if let Some(liquidation_price) = self.liquidation_price {
            match self.side {
                PositionSide::Long => self.mark_price <= liquidation_price,
                PositionSide::Short => self.mark_price >= liquidation_price,
            }
        } else {
            self.margin_ratio <= maintenance_margin_rate
        }
    }

    /// 部分平仓
    pub fn partial_close(
        &mut self,
        close_size: Quantity,
        close_price: Price,
    ) -> TradingResult<Amount> {
        if close_size <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Close size must be positive".to_string(),
            ));
        }

        if close_size > self.size {
            return Err(TradingError::InvalidOrder(
                "Close size exceeds position size".to_string(),
            ));
        }

        if close_price <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Close price must be positive".to_string(),
            ));
        }

        // 计算平仓盈亏
        let close_pnl = match self.side {
            PositionSide::Long => (close_price - self.entry_price) * close_size,
            PositionSide::Short => (self.entry_price - close_price) * close_size,
        };

        // 更新仓位
        self.size -= close_size;
        self.realized_pnl += close_pnl;

        // 按比例释放保证金
        let margin_ratio = close_size / (self.size + close_size);
        let released_margin = self.margin * margin_ratio;
        self.margin -= released_margin;

        self.updated_at = Utc::now();

        // 如果仓位完全平仓
        if self.size == Decimal::ZERO {
            self.status = PositionStatus::Closed;
            self.closed_at = Some(Utc::now());
        }

        Ok(close_pnl)
    }

    /// 完全平仓
    pub fn close(&mut self, close_price: Price) -> TradingResult<Amount> {
        self.partial_close(self.size, close_price)
    }

    /// 增加仓位
    pub fn increase_position(
        &mut self,
        add_size: Quantity,
        add_price: Price,
        add_margin: Amount,
    ) -> TradingResult<()> {
        if add_size <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Add size must be positive".to_string(),
            ));
        }

        if add_price <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Add price must be positive".to_string(),
            ));
        }

        if add_margin <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Add margin must be positive".to_string(),
            ));
        }

        // 计算新的平均入场价
        let total_value = (self.entry_price * self.size) + (add_price * add_size);
        let total_size = self.size + add_size;
        self.entry_price = total_value / total_size;

        // 更新仓位信息
        self.size = total_size;
        self.margin += add_margin;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 获取仓位价值
    pub fn get_position_value(&self) -> Amount {
        self.mark_price * self.size
    }

    /// 获取总盈亏
    pub fn get_total_pnl(&self) -> Amount {
        self.realized_pnl + self.unrealized_pnl
    }

    /// 获取投资回报率
    pub fn get_roi(&self) -> Decimal {
        if self.margin > Decimal::ZERO {
            self.get_total_pnl() / self.margin * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }

    /// 检查仓位是否健康
    pub fn is_healthy(&self, maintenance_margin_rate: Decimal) -> bool {
        self.margin_ratio > maintenance_margin_rate
            && !self.should_liquidate(maintenance_margin_rate)
    }
}

/// 仓位摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSummary {
    pub symbol: Symbol,
    pub side: PositionSide,
    pub size: Quantity,
    pub entry_price: Price,
    pub mark_price: Price,
    pub unrealized_pnl: Amount,
    pub margin_ratio: Decimal,
    pub roi: Decimal,
}

impl From<&Position> for PositionSummary {
    fn from(position: &Position) -> Self {
        Self {
            symbol: position.symbol.clone(),
            side: position.side,
            size: position.size,
            entry_price: position.entry_price,
            mark_price: position.mark_price,
            unrealized_pnl: position.unrealized_pnl,
            margin_ratio: position.margin_ratio,
            roi: position.get_roi(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        let position = Position::new(
            user_id,
            symbol,
            PositionSide::Long,
            Decimal::from(1),
            Decimal::from(50000),
            Decimal::from(10),
            Decimal::from(5000),
        )
        .unwrap();

        assert_eq!(position.user_id, user_id);
        assert_eq!(position.side, PositionSide::Long);
        assert_eq!(position.size, Decimal::from(1));
        assert_eq!(position.entry_price, Decimal::from(50000));
        assert_eq!(position.leverage, Decimal::from(10));
        assert_eq!(position.margin, Decimal::from(5000));
        assert_eq!(position.status, PositionStatus::Open);
    }

    #[test]
    fn test_unrealized_pnl_calculation() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        let mut position = Position::new(
            user_id,
            symbol,
            PositionSide::Long,
            Decimal::from(1),
            Decimal::from(50000),
            Decimal::from(10),
            Decimal::from(5000),
        )
        .unwrap();

        // 价格上涨，多头盈利
        position.update_mark_price(Decimal::from(51000)).unwrap();
        assert_eq!(position.unrealized_pnl, Decimal::from(1000));

        // 价格下跌，多头亏损
        position.update_mark_price(Decimal::from(49000)).unwrap();
        assert_eq!(position.unrealized_pnl, Decimal::from(-1000));
    }

    #[test]
    fn test_position_close() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        let mut position = Position::new(
            user_id,
            symbol,
            PositionSide::Long,
            Decimal::from(2),
            Decimal::from(50000),
            Decimal::from(10),
            Decimal::from(10000),
        )
        .unwrap();

        // 部分平仓
        let pnl = position
            .partial_close(Decimal::from(1), Decimal::from(51000))
            .unwrap();
        assert_eq!(pnl, Decimal::from(1000)); // (51000 - 50000) * 1
        assert_eq!(position.size, Decimal::from(1));
        assert_eq!(position.realized_pnl, Decimal::from(1000));
        assert_eq!(position.status, PositionStatus::Open);

        // 完全平仓
        let pnl = position.close(Decimal::from(52000)).unwrap();
        assert_eq!(pnl, Decimal::from(2000)); // (52000 - 50000) * 1
        assert_eq!(position.size, Decimal::ZERO);
        assert_eq!(position.realized_pnl, Decimal::from(3000));
        assert_eq!(position.status, PositionStatus::Closed);
        assert!(position.closed_at.is_some());
    }

    #[test]
    fn test_liquidation_price_calculation() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        let mut position = Position::new(
            user_id,
            symbol,
            PositionSide::Long,
            Decimal::from(1),
            Decimal::from(50000),
            Decimal::from(10),
            Decimal::from(5000),
        )
        .unwrap();

        let maintenance_margin_rate = Decimal::new(5, 2); // 5%
        position.update_liquidation_price(maintenance_margin_rate);

        // 对于多头仓位，强平价格应该低于入场价格
        assert!(position.liquidation_price.unwrap() < position.entry_price);
    }
}
