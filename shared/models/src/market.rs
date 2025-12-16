use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::{DataQuality, Exchange, Interval};

/// 市场Tick数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub id: Option<Uuid>,
    pub exchange: Exchange,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub volume: Decimal,
    pub bid: Decimal,
    pub ask: Decimal,
    pub bid_volume: Decimal,
    pub ask_volume: Decimal,
    pub trade_id: Option<String>,
    pub is_buyer_maker: Option<bool>,
    /// 数据质量标记
    pub data_quality: DataQuality,
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub id: Option<Uuid>,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: Interval,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub trades_count: u32,
    pub taker_buy_base_volume: Decimal,
    pub taker_buy_quote_volume: Decimal,
    pub is_closed: bool,
    /// 数据质量标记
    pub data_quality: DataQuality,
}

/// 订单簿数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub exchange: Exchange,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub last_update_id: u64,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

/// 订单簿价格层级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// 24小时统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker24hr {
    pub exchange: Exchange,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
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
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub first_id: u64,
    pub last_id: u64,
    pub count: u32,
}

/// 市场数据订阅请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub symbol: String,
    pub data_type: MarketDataType,
    pub interval: Option<Interval>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// 市场数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketDataType {
    Tick,
    Kline,
    OrderBook,
    Ticker24hr,
    Trade,
}

/// 交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Option<Uuid>,
    pub exchange: Exchange,
    pub symbol: String,
    pub trade_id: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub quantity: Decimal,
    pub quote_quantity: Decimal,
    pub side: String, // "buy" or "sell"
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

/// 市场数据查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataQuery {
    pub exchange: Option<Exchange>,
    pub symbol: Option<String>,
    pub interval: Option<Interval>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

/// 实时价格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub volume_24h: Decimal,
    pub change_24h: Decimal,
    pub change_percent_24h: Decimal,
}

/// 市场深度快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthSnapshot {
    pub exchange: Exchange,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub last_update_id: u64,
}

/// WebSocket市场数据消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataMessage {
    pub message_type: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// 市场数据统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataStats {
    pub exchange: Exchange,
    pub symbol: String,
    pub date: chrono::NaiveDate,
    pub total_trades: u64,
    pub total_volume: Decimal,
    pub total_quote_volume: Decimal,
    pub avg_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub open_price: Decimal,
    pub close_price: Decimal,
}
