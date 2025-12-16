use anyhow::Result;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Order, Symbol, Side, OrderType};

/// 统一交易所接口 - 支持所有主流交易所
#[async_trait]
pub trait ExchangeConnector: Send + Sync {
    /// 交易所基本信息
    fn get_exchange_info(&self) -> ExchangeInfo;
    
    /// 连接管理
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn is_connected(&self) -> bool;
    async fn reconnect(&mut self) -> Result<()>;
    
    /// 账户管理
    async fn get_account_info(&self) -> Result<AccountInfo>;
    async fn get_balances(&self) -> Result<Vec<Balance>>;
    async fn get_trading_fees(&self, symbol: &Symbol) -> Result<TradingFees>;
    
    /// 市场数据
    async fn get_ticker(&self, symbol: &Symbol) -> Result<Ticker>;
    async fn get_order_book(&self, symbol: &Symbol, limit: Option<u32>) -> Result<OrderBook>;
    async fn get_klines(&self, symbol: &Symbol, interval: &str, limit: Option<u32>) -> Result<Vec<Kline>>;
    async fn get_24hr_stats(&self, symbol: &Symbol) -> Result<Stats24Hr>;
    
    /// 订单管理
    async fn place_order(&self, order: &UnifiedOrder) -> Result<OrderResponse>;
    async fn cancel_order(&self, symbol: &Symbol, order_id: &str) -> Result<CancelResponse>;
    async fn cancel_all_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<CancelResponse>>;
    async fn get_order(&self, symbol: &Symbol, order_id: &str) -> Result<OrderInfo>;
    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<OrderInfo>>;
    async fn get_order_history(&self, symbol: Option<&Symbol>, limit: Option<u32>) -> Result<Vec<OrderInfo>>;
    
    /// 仓位管理 (期货)
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn get_position(&self, symbol: &Symbol) -> Result<Option<Position>>;
    async fn set_leverage(&self, symbol: &Symbol, leverage: u32) -> Result<()>;
    async fn set_margin_mode(&self, symbol: &Symbol, mode: MarginMode) -> Result<()>;
    
    /// 交易历史
    async fn get_trade_history(&self, symbol: Option<&Symbol>, limit: Option<u32>) -> Result<Vec<Trade>>;
    
    /// WebSocket 流
    async fn subscribe_ticker(&self, symbols: Vec<Symbol>) -> Result<()>;
    async fn subscribe_order_book(&self, symbols: Vec<Symbol>) -> Result<()>;
    async fn subscribe_trades(&self, symbols: Vec<Symbol>) -> Result<()>;
    async fn subscribe_user_data(&self) -> Result<()>;
    
    /// 高级功能
    async fn get_exchange_time(&self) -> Result<i64>;
    async fn get_symbols(&self) -> Result<Vec<SymbolInfo>>;
    async fn get_exchange_status(&self) -> Result<ExchangeStatus>;
    
    /// 批量操作
    async fn place_batch_orders(&self, orders: Vec<UnifiedOrder>) -> Result<Vec<OrderResponse>>;
    async fn cancel_batch_orders(&self, orders: Vec<(Symbol, String)>) -> Result<Vec<CancelResponse>>;
}

/// 交易所信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    pub display_name: String,
    pub country: String,
    pub website: String,
    pub api_version: String,
    pub supports_spot: bool,
    pub supports_futures: bool,
    pub supports_options: bool,
    pub supports_margin: bool,
    pub rate_limits: RateLimits,
    pub trading_fees: DefaultFees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub orders_per_second: u32,
    pub orders_per_minute: u32,
    pub weight_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultFees {
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub withdrawal_fee: HashMap<String, Decimal>,
}

/// 统一订单结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedOrder {
    pub client_order_id: Option<String>,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub time_in_force: Option<TimeInForce>,
    pub reduce_only: Option<bool>,
    pub post_only: Option<bool>,
    pub iceberg_qty: Option<Decimal>,
    pub strategy_id: Option<Uuid>,
    pub strategy_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    GTD, // Good Till Date
}

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub account_type: String,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub update_time: i64,
    pub balances: Vec<Balance>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingFees {
    pub symbol: Symbol,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
}

/// 市场数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: Symbol,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub weighted_avg_price: Decimal,
    pub prev_close_price: Decimal,
    pub last_price: Decimal,
    pub last_qty: Decimal,
    pub bid_price: Decimal,
    pub bid_qty: Decimal,
    pub ask_price: Decimal,
    pub ask_qty: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub open_time: i64,
    pub close_time: i64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub last_update_id: u64,
    pub bids: Vec<(Decimal, Decimal)>, // (price, quantity)
    pub asks: Vec<(Decimal, Decimal)>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub open_time: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub close_time: i64,
    pub quote_asset_volume: Decimal,
    pub number_of_trades: u64,
    pub taker_buy_base_asset_volume: Decimal,
    pub taker_buy_quote_asset_volume: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats24Hr {
    pub symbol: Symbol,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub weighted_avg_price: Decimal,
    pub prev_close_price: Decimal,
    pub last_price: Decimal,
    pub last_qty: Decimal,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub open_time: i64,
    pub close_time: i64,
    pub first_id: u64,
    pub last_id: u64,
    pub count: u64,
}

/// 订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub symbol: Symbol,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub transact_time: i64,
    pub price: Option<Decimal>,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: Option<TimeInForce>,
    pub order_type: OrderType,
    pub side: Side,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub price: Decimal,
    pub qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub trade_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    PendingCancel,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    pub symbol: Symbol,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInfo {
    pub symbol: Symbol,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub price: Option<Decimal>,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: Option<TimeInForce>,
    pub order_type: OrderType,
    pub side: Side,
    pub stop_price: Option<Decimal>,
    pub iceberg_qty: Option<Decimal>,
    pub time: i64,
    pub update_time: i64,
    pub is_working: bool,
    pub orig_quote_order_qty: Option<Decimal>,
}

/// 仓位信息 (期货)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: Symbol,
    pub position_amt: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub liquidation_price: Option<Decimal>,
    pub leverage: u32,
    pub max_notional_value: Decimal,
    pub margin_type: MarginMode,
    pub isolated_margin: Option<Decimal>,
    pub is_auto_add_margin: bool,
    pub position_side: PositionSide,
    pub notional: Decimal,
    pub isolated_wallet: Option<Decimal>,
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarginMode {
    Isolated,
    Cross,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSide {
    Both,
    Long,
    Short,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: Symbol,
    pub id: String,
    pub order_id: String,
    pub order_list_id: Option<String>,
    pub price: Decimal,
    pub qty: Decimal,
    pub quote_qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub time: i64,
    pub is_buyer: bool,
    pub is_maker: bool,
    pub is_best_match: bool,
}

/// 交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: Symbol,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: u32,
    pub quote_asset: String,
    pub quote_precision: u32,
    pub quote_asset_precision: u32,
    pub order_types: Vec<OrderType>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<SymbolFilter>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFilter {
    pub filter_type: String,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub tick_size: Option<Decimal>,
    pub min_qty: Option<Decimal>,
    pub max_qty: Option<Decimal>,
    pub step_size: Option<Decimal>,
    pub min_notional: Option<Decimal>,
    pub apply_to_market: Option<bool>,
    pub avg_price_mins: Option<u32>,
    pub limit: Option<u32>,
    pub max_num_orders: Option<u32>,
    pub max_num_algo_orders: Option<u32>,
}

/// 交易所状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeStatus {
    pub status: String,
    pub message: Option<String>,
}

/// WebSocket 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketEvent {
    Ticker(Ticker),
    OrderBook(OrderBook),
    Trade(Trade),
    Kline(Kline),
    UserData(UserDataEvent),
    Error(String),
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserDataEvent {
    AccountUpdate(AccountInfo),
    BalanceUpdate(Balance),
    OrderUpdate(OrderInfo),
    TradeUpdate(Trade),
    PositionUpdate(Position),
}

/// 交易所错误类型
#[derive(Debug, thiserror::Error)]
pub enum ExchangeError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    
    #[error("Invalid order: {0}")]
    InvalidOrder(String),
    
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("Market closed: {0}")]
    MarketClosed(String),
    
    #[error("API error: {code} - {message}")]
    ApiError { code: i32, message: String },
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type ExchangeResult<T> = Result<T, ExchangeError>;