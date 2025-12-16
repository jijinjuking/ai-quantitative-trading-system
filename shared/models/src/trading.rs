use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::common::Exchange;

/// 订单结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub symbol: String,
    pub client_order_id: String,
    pub exchange_order_id: Option<String>,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub iceberg_qty: Option<Decimal>,
    pub status: OrderStatus,
    pub filled_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub avg_price: Option<Decimal>,
    pub commission: Decimal,
    pub commission_asset: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 订单方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

/// 订单类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

/// 订单有效期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    GTX, // Good Till Crossing
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,       // 待提交
    Submitted,     // 已提交
    PartialFilled, // 部分成交
    Filled,        // 完全成交
    Cancelled,     // 已取消
    Rejected,      // 被拒绝
    Failed,        // 执行失败
    Expired,       // 已过期
}

/// 交易执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub symbol: String,
    pub trade_id: String,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Decimal,
    pub quote_quantity: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub is_maker: bool,
    pub executed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// 仓位信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub symbol: String,
    pub side: PositionSide,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub margin: Decimal,
    pub leverage: Decimal,
    pub liquidation_price: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 仓位方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
    Both, // 双向持仓模式
}

/// 账户余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
    pub updated_at: DateTime<Utc>,
}

/// 交易账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub account_type: AccountType,
    pub status: AccountStatus,
    pub balances: Vec<Balance>,
    pub total_wallet_balance: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub total_margin_balance: Decimal,
    pub total_position_initial_margin: Decimal,
    pub total_open_order_initial_margin: Decimal,
    pub max_withdraw_amount: Decimal,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub updated_at: DateTime<Utc>,
}

/// 账户类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Spot,
    Margin,
    Futures,
    Options,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Spot => write!(f, "SPOT"),
            AccountType::Margin => write!(f, "MARGIN"),
            AccountType::Futures => write!(f, "FUTURES"),
            AccountType::Options => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SPOT" => Ok(AccountType::Spot),
            "MARGIN" => Ok(AccountType::Margin),
            "FUTURES" => Ok(AccountType::Futures),
            "OPTIONS" => Ok(AccountType::Options),
            _ => Err(anyhow::anyhow!("Invalid account type: {}", s)),
        }
    }
}

/// 账户状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Suspended,
    Restricted,
    Closed,
}

/// 订单创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: Option<TimeInForce>,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub iceberg_qty: Option<Decimal>,
    pub client_order_id: Option<String>,
    pub reduce_only: Option<bool>,
    pub close_position: Option<bool>,
}

/// 订单修改请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyOrderRequest {
    pub order_id: Uuid,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
}

/// 订单取消请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: Option<Uuid>,
    pub client_order_id: Option<String>,
    pub symbol: Option<String>,
}

/// 批量订单操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOrderRequest {
    pub orders: Vec<CreateOrderRequest>,
}

/// 交易统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStats {
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub symbol: Option<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_trades: u64,
    pub total_volume: Decimal,
    pub total_quote_volume: Decimal,
    pub total_commission: Decimal,
    pub realized_pnl: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
    pub sharpe_ratio: Option<Decimal>,
}

/// 交易信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub id: Uuid,
    pub strategy_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub signal_type: SignalType,
    pub strength: Decimal,
    pub price: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 信号类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Entry,
    Exit,
    StopLoss,
    TakeProfit,
    PositionSize,
}

/// 执行报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub order_id: Uuid,
    pub client_order_id: String,
    pub exchange_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub avg_price: Option<Decimal>,
    pub commission: Decimal,
    pub commission_asset: String,
    pub timestamp: DateTime<Utc>,
    pub execution_type: ExecutionType,
}

/// 执行类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionType {
    New,
    Cancelled,
    Replaced,
    Rejected,
    Trade,
    Expired,
}
