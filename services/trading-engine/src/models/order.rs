use super::{Amount, Id, Price, Quantity, Side, Symbol, Timestamp, TradingError, TradingResult};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    StopLossLimit,
    TakeProfitLimit,
}

impl OrderType {
    /// 是否需要价格参数
    pub fn requires_price(&self) -> bool {
        matches!(
            self,
            OrderType::Limit | OrderType::StopLossLimit | OrderType::TakeProfitLimit
        )
    }

    /// 是否需要止损价格参数
    pub fn requires_stop_price(&self) -> bool {
        matches!(
            self,
            OrderType::StopLoss
                | OrderType::TakeProfit
                | OrderType::StopLossLimit
                | OrderType::TakeProfitLimit
        )
    }

    /// 是否为市价单
    pub fn is_market_order(&self) -> bool {
        matches!(self, OrderType::Market)
    }

    /// 是否为限价单
    pub fn is_limit_order(&self) -> bool {
        matches!(
            self,
            OrderType::Limit | OrderType::StopLossLimit | OrderType::TakeProfitLimit
        )
    }
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "MARKET"),
            OrderType::Limit => write!(f, "LIMIT"),
            OrderType::StopLoss => write!(f, "STOP_LOSS"),
            OrderType::TakeProfit => write!(f, "TAKE_PROFIT"),
            OrderType::StopLossLimit => write!(f, "STOP_LOSS_LIMIT"),
            OrderType::TakeProfitLimit => write!(f, "TAKE_PROFIT_LIMIT"),
        }
    }
}

impl std::str::FromStr for OrderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MARKET" => Ok(OrderType::Market),
            "LIMIT" => Ok(OrderType::Limit),
            "STOP_LOSS" => Ok(OrderType::StopLoss),
            "TAKE_PROFIT" => Ok(OrderType::TakeProfit),
            "STOP_LOSS_LIMIT" => Ok(OrderType::StopLossLimit),
            "TAKE_PROFIT_LIMIT" => Ok(OrderType::TakeProfitLimit),
            _ => Err(anyhow::anyhow!("Invalid order type: {}", s)),
        }
    }
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl OrderStatus {
    /// 是否为活跃状态
    pub fn is_active(&self) -> bool {
        matches!(self, OrderStatus::Pending | OrderStatus::PartiallyFilled)
    }

    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Rejected
                | OrderStatus::Expired
        )
    }

    /// 是否可以取消
    pub fn can_cancel(&self) -> bool {
        matches!(self, OrderStatus::Pending | OrderStatus::PartiallyFilled)
    }

    /// 是否可以修改
    pub fn can_modify(&self) -> bool {
        matches!(self, OrderStatus::Pending)
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "PENDING"),
            OrderStatus::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            OrderStatus::Filled => write!(f, "FILLED"),
            OrderStatus::Cancelled => write!(f, "CANCELLED"),
            OrderStatus::Rejected => write!(f, "REJECTED"),
            OrderStatus::Expired => write!(f, "EXPIRED"),
        }
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PENDING" => Ok(OrderStatus::Pending),
            "PARTIALLY_FILLED" => Ok(OrderStatus::PartiallyFilled),
            "FILLED" => Ok(OrderStatus::Filled),
            "CANCELLED" => Ok(OrderStatus::Cancelled),
            "REJECTED" => Ok(OrderStatus::Rejected),
            "EXPIRED" => Ok(OrderStatus::Expired),
            _ => Err(anyhow::anyhow!("Invalid order status: {}", s)),
        }
    }
}

/// 订单时效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    GTD, // Good Till Date
}

impl std::fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInForce::GTC => write!(f, "GTC"),
            TimeInForce::IOC => write!(f, "IOC"),
            TimeInForce::FOK => write!(f, "FOK"),
            TimeInForce::GTD => write!(f, "GTD"),
        }
    }
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Id,
    pub user_id: Id,
    pub symbol: Symbol,
    pub order_type: OrderType,
    pub side: Side,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub stop_price: Option<Price>,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub filled_quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub average_price: Option<Price>,
    pub fee: Amount,
    pub fee_currency: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub client_order_id: Option<String>,
    pub metadata: OrderMetadata,
}

/// 订单元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMetadata {
    pub source: String,
    pub algorithm: Option<String>,
    pub parent_order_id: Option<Id>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

impl Default for OrderMetadata {
    fn default() -> Self {
        Self {
            source: "api".to_string(),
            algorithm: None,
            parent_order_id: None,
            tags: Vec::new(),
            notes: None,
        }
    }
}

impl Order {
    /// 创建新订单
    pub fn new(
        user_id: Id,
        symbol: Symbol,
        order_type: OrderType,
        side: Side,
        quantity: Quantity,
        price: Option<Price>,
        stop_price: Option<Price>,
    ) -> TradingResult<Self> {
        // 验证订单参数
        if quantity <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Quantity must be positive".to_string(),
            ));
        }

        if order_type.requires_price() && price.is_none() {
            return Err(TradingError::InvalidOrder(
                "Price is required for this order type".to_string(),
            ));
        }

        if order_type.requires_stop_price() && stop_price.is_none() {
            return Err(TradingError::InvalidOrder(
                "Stop price is required for this order type".to_string(),
            ));
        }

        if let Some(p) = price {
            if p <= Decimal::ZERO {
                return Err(TradingError::InvalidOrder(
                    "Price must be positive".to_string(),
                ));
            }
        }

        if let Some(sp) = stop_price {
            if sp <= Decimal::ZERO {
                return Err(TradingError::InvalidOrder(
                    "Stop price must be positive".to_string(),
                ));
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            symbol,
            order_type,
            side,
            quantity,
            price,
            stop_price,
            status: OrderStatus::Pending,
            time_in_force: TimeInForce::GTC,
            filled_quantity: Decimal::ZERO,
            remaining_quantity: quantity,
            average_price: None,
            fee: Decimal::ZERO,
            fee_currency: "USDT".to_string(),
            created_at: now,
            updated_at: now,
            expires_at: None,
            client_order_id: None,
            metadata: OrderMetadata::default(),
        })
    }

    /// 设置时效类型
    pub fn with_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// 设置过期时间
    pub fn with_expiry(mut self, expires_at: Timestamp) -> Self {
        self.expires_at = Some(expires_at);
        self.time_in_force = TimeInForce::GTD;
        self
    }

    /// 设置客户端订单ID
    pub fn with_client_order_id(mut self, client_order_id: String) -> Self {
        self.client_order_id = Some(client_order_id);
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: OrderMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// 检查订单是否有效
    pub fn validate(&self) -> TradingResult<()> {
        if self.quantity <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Quantity must be positive".to_string(),
            ));
        }

        if self.order_type.requires_price() && self.price.is_none() {
            return Err(TradingError::InvalidOrder("Price is required".to_string()));
        }

        if self.order_type.requires_stop_price() && self.stop_price.is_none() {
            return Err(TradingError::InvalidOrder(
                "Stop price is required".to_string(),
            ));
        }

        if self.filled_quantity > self.quantity {
            return Err(TradingError::InvalidOrder(
                "Filled quantity cannot exceed total quantity".to_string(),
            ));
        }

        if self.remaining_quantity != self.quantity - self.filled_quantity {
            return Err(TradingError::InvalidOrder(
                "Remaining quantity mismatch".to_string(),
            ));
        }

        Ok(())
    }

    /// 获取填充百分比
    pub fn fill_percentage(&self) -> Decimal {
        if self.quantity > Decimal::ZERO {
            self.filled_quantity / self.quantity * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }

    /// 检查是否完全成交
    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }

    /// 检查是否部分成交
    pub fn is_partially_filled(&self) -> bool {
        self.filled_quantity > Decimal::ZERO && self.filled_quantity < self.quantity
    }

    /// 检查是否未成交
    pub fn is_unfilled(&self) -> bool {
        self.filled_quantity == Decimal::ZERO
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 计算订单价值
    pub fn calculate_value(&self) -> Option<Amount> {
        match self.order_type {
            OrderType::Market => {
                // 市价单使用平均成交价或当前市价估算
                self.average_price.map(|p| p * self.quantity)
            }
            _ => {
                // 限价单使用设定价格
                self.price.map(|p| p * self.quantity)
            }
        }
    }

    /// 计算已成交价值
    pub fn calculate_filled_value(&self) -> Amount {
        if let Some(avg_price) = self.average_price {
            avg_price * self.filled_quantity
        } else {
            Decimal::ZERO
        }
    }

    /// 更新成交信息
    pub fn update_fill(
        &mut self,
        fill_quantity: Quantity,
        fill_price: Price,
        fee: Amount,
    ) -> TradingResult<()> {
        if fill_quantity <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Fill quantity must be positive".to_string(),
            ));
        }

        if fill_price <= Decimal::ZERO {
            return Err(TradingError::InvalidOrder(
                "Fill price must be positive".to_string(),
            ));
        }

        if self.filled_quantity + fill_quantity > self.quantity {
            return Err(TradingError::InvalidOrder(
                "Fill quantity exceeds remaining quantity".to_string(),
            ));
        }

        // 更新平均价格
        let total_filled_value = self.calculate_filled_value() + (fill_price * fill_quantity);
        let new_filled_quantity = self.filled_quantity + fill_quantity;
        self.average_price = Some(total_filled_value / new_filled_quantity);

        // 更新数量
        self.filled_quantity = new_filled_quantity;
        self.remaining_quantity = self.quantity - self.filled_quantity;

        // 更新手续费
        self.fee += fee;

        // 更新状态
        if self.is_fully_filled() {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }

        self.updated_at = Utc::now();

        Ok(())
    }

    /// 取消订单
    pub fn cancel(&mut self) -> TradingResult<()> {
        if !self.status.can_cancel() {
            return Err(TradingError::InvalidOrder(format!(
                "Cannot cancel order in status: {}",
                self.status
            )));
        }

        self.status = OrderStatus::Cancelled;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 拒绝订单
    pub fn reject(&mut self, reason: &str) -> TradingResult<()> {
        if self.status != OrderStatus::Pending {
            return Err(TradingError::InvalidOrder(format!(
                "Cannot reject order in status: {}",
                self.status
            )));
        }

        self.status = OrderStatus::Rejected;
        self.metadata.notes = Some(reason.to_string());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 标记为过期
    pub fn expire(&mut self) -> TradingResult<()> {
        if !self.status.is_active() {
            return Err(TradingError::InvalidOrder(format!(
                "Cannot expire order in status: {}",
                self.status
            )));
        }

        self.status = OrderStatus::Expired;
        self.updated_at = Utc::now();

        Ok(())
    }
}

/// 订单创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub order_type: String,
    pub side: String,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub stop_price: Option<Price>,
    pub time_in_force: Option<String>,
    pub expires_at: Option<Timestamp>,
    pub client_order_id: Option<String>,
}

impl CreateOrderRequest {
    /// 转换为订单
    pub fn to_order(&self, user_id: Id) -> TradingResult<Order> {
        let symbol = self
            .symbol
            .parse()
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid symbol: {}", e)))?;

        let order_type = self
            .order_type
            .parse()
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid order type: {}", e)))?;

        let side = self
            .side
            .parse()
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid side: {}", e)))?;

        let mut order = Order::new(
            user_id,
            symbol,
            order_type,
            side,
            self.quantity,
            self.price,
            self.stop_price,
        )?;

        if let Some(tif_str) = &self.time_in_force {
            let tif = match tif_str.to_uppercase().as_str() {
                "GTC" => TimeInForce::GTC,
                "IOC" => TimeInForce::IOC,
                "FOK" => TimeInForce::FOK,
                "GTD" => TimeInForce::GTD,
                _ => {
                    return Err(TradingError::InvalidOrder(format!(
                        "Invalid time in force: {}",
                        tif_str
                    )))
                }
            };
            order = order.with_time_in_force(tif);
        }

        if let Some(expires_at) = self.expires_at {
            order = order.with_expiry(expires_at);
        }

        if let Some(client_order_id) = &self.client_order_id {
            order = order.with_client_order_id(client_order_id.clone());
        }

        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        let order = Order::new(
            user_id,
            symbol,
            OrderType::Limit,
            Side::Buy,
            Decimal::from(1),
            Some(Decimal::from(50000)),
            None,
        )
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.remaining_quantity, Decimal::from(1));
    }

    #[test]
    fn test_order_fill() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        let mut order = Order::new(
            user_id,
            symbol,
            OrderType::Limit,
            Side::Buy,
            Decimal::from(2),
            Some(Decimal::from(50000)),
            None,
        )
        .unwrap();

        // 部分成交
        order
            .update_fill(Decimal::from(1), Decimal::from(49900), Decimal::from(10))
            .unwrap();
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_quantity, Decimal::from(1));
        assert_eq!(order.remaining_quantity, Decimal::from(1));
        assert_eq!(order.average_price, Some(Decimal::from(49900)));

        // 完全成交
        order
            .update_fill(Decimal::from(1), Decimal::from(50100), Decimal::from(10))
            .unwrap();
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_quantity, Decimal::from(2));
        assert_eq!(order.remaining_quantity, Decimal::ZERO);
        assert_eq!(order.average_price, Some(Decimal::from(50000))); // (49900 + 50100) / 2
    }

    #[test]
    fn test_order_validation() {
        let user_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC", "USDT");

        // 无效数量
        let result = Order::new(
            user_id,
            symbol.clone(),
            OrderType::Market,
            Side::Buy,
            Decimal::ZERO,
            None,
            None,
        );
        assert!(result.is_err());

        // 限价单缺少价格
        let result = Order::new(
            user_id,
            symbol.clone(),
            OrderType::Limit,
            Side::Buy,
            Decimal::from(1),
            None,
            None,
        );
        assert!(result.is_err());

        // 止损单缺少止损价
        let result = Order::new(
            user_id,
            symbol,
            OrderType::StopLoss,
            Side::Sell,
            Decimal::from(1),
            None,
            None,
        );
        assert!(result.is_err());
    }
}
