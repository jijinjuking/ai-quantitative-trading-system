pub mod account;
pub mod order;
pub mod position;

pub use account::*;
pub use order::*;
pub use position::*;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 通用ID类型
pub type Id = Uuid;

/// 通用时间戳类型
pub type Timestamp = DateTime<Utc>;

/// 通用价格类型
pub type Price = Decimal;

/// 通用数量类型
pub type Quantity = Decimal;

/// 通用金额类型
pub type Amount = Decimal;

/// 交易对
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
}

impl Symbol {
    pub fn new(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    pub fn from_string(symbol: &str) -> Option<Self> {
        // 尝试解析常见格式: BTCUSDT, BTC/USDT, BTC-USDT
        if symbol.len() >= 6 {
            // 尝试BTCUSDT格式
            if symbol.ends_with("USDT") && symbol.len() > 4 {
                let base = &symbol[..symbol.len() - 4];
                return Some(Self::new(base, "USDT"));
            }
            if symbol.ends_with("BTC") && symbol.len() > 3 {
                let base = &symbol[..symbol.len() - 3];
                return Some(Self::new(base, "BTC"));
            }
            if symbol.ends_with("ETH") && symbol.len() > 3 {
                let base = &symbol[..symbol.len() - 3];
                return Some(Self::new(base, "ETH"));
            }
        }

        // 尝试分隔符格式
        if let Some(pos) = symbol.find('/') {
            let base = &symbol[..pos];
            let quote = &symbol[pos + 1..];
            return Some(Self::new(base, quote));
        }

        if let Some(pos) = symbol.find('-') {
            let base = &symbol[..pos];
            let quote = &symbol[pos + 1..];
            return Some(Self::new(base, quote));
        }

        None
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }

    pub fn to_string_with_separator(&self, separator: &str) -> String {
        format!("{}{}{}", self.base, separator, self.quote)
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for Symbol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s).ok_or_else(|| anyhow::anyhow!("Invalid symbol format: {}", s))
    }
}

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }

    pub fn is_buy(&self) -> bool {
        matches!(self, Side::Buy)
    }

    pub fn is_sell(&self) -> bool {
        matches!(self, Side::Sell)
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

impl std::str::FromStr for Side {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BUY" | "B" => Ok(Side::Buy),
            "SELL" | "S" => Ok(Side::Sell),
            _ => Err(anyhow::anyhow!("Invalid side: {}", s)),
        }
    }
}

/// 市场数据快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: Symbol,
    pub bid_price: Option<Price>,
    pub ask_price: Option<Price>,
    pub last_price: Option<Price>,
    pub volume_24h: Quantity,
    pub price_change_24h: Decimal,
    pub timestamp: Timestamp,
}

impl MarketData {
    /// 获取中间价
    pub fn mid_price(&self) -> Option<Price> {
        match (self.bid_price, self.ask_price) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => self.last_price,
        }
    }

    /// 获取买卖价差
    pub fn spread(&self) -> Option<Price> {
        match (self.bid_price, self.ask_price) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// 获取买卖价差百分比
    pub fn spread_percentage(&self) -> Option<Decimal> {
        match (self.spread(), self.mid_price()) {
            (Some(spread), Some(mid)) if mid > Decimal::ZERO => {
                Some(spread / mid * Decimal::from(100))
            }
            _ => None,
        }
    }
}

/// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum TradingError {
    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: Amount, available: Amount },

    #[error("Insufficient margin: required {required}, available {available}")]
    InsufficientMargin { required: Amount, available: Amount },

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Order not found: {0}")]
    OrderNotFound(Uuid),

    #[error("Risk violation: {0}")]
    RiskViolation(String),

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Market closed for symbol: {0}")]
    MarketClosed(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type TradingResult<T> = Result<T, TradingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_parsing() {
        // 测试标准格式
        let symbol = Symbol::from_string("BTCUSDT").unwrap();
        assert_eq!(symbol.base, "BTC");
        assert_eq!(symbol.quote, "USDT");

        // 测试分隔符格式
        let symbol = Symbol::from_string("BTC/USDT").unwrap();
        assert_eq!(symbol.base, "BTC");
        assert_eq!(symbol.quote, "USDT");

        let symbol = Symbol::from_string("BTC-USDT").unwrap();
        assert_eq!(symbol.base, "BTC");
        assert_eq!(symbol.quote, "USDT");

        // 测试字符串转换
        let symbol = Symbol::new("btc", "usdt");
        assert_eq!(symbol.to_string(), "BTCUSDT");
        assert_eq!(symbol.to_string_with_separator("/"), "BTC/USDT");
    }

    #[test]
    fn test_side_operations() {
        assert_eq!(Side::Buy.opposite(), Side::Sell);
        assert_eq!(Side::Sell.opposite(), Side::Buy);

        assert!(Side::Buy.is_buy());
        assert!(!Side::Buy.is_sell());

        assert!(Side::Sell.is_sell());
        assert!(!Side::Sell.is_buy());
    }

    #[test]
    fn test_market_data_calculations() {
        let market_data = MarketData {
            symbol: Symbol::new("BTC", "USDT"),
            bid_price: Some(Decimal::from(50000)),
            ask_price: Some(Decimal::from(50100)),
            last_price: Some(Decimal::from(50050)),
            volume_24h: Decimal::from(1000),
            price_change_24h: Decimal::from(500),
            timestamp: Utc::now(),
        };

        assert_eq!(market_data.mid_price(), Some(Decimal::from(50050)));
        assert_eq!(market_data.spread(), Some(Decimal::from(100)));

        let spread_pct = market_data.spread_percentage().unwrap();
        assert!((spread_pct - Decimal::new(1998, 4)).abs() < Decimal::new(1, 6));
        // ~0.1998%
    }
}
