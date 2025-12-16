pub mod binance;
pub mod exchange_manager;
pub mod websocket_client;
pub mod connection_pool;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use shared_models::market::{MarketTick, Kline, OrderBook, Trade};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub use binance::BinanceConnector;
pub use exchange_manager::ExchangeManager;
pub use websocket_client::WebSocketClient;
pub use connection_pool::ConnectionPool;

/// 交易所连接器特征
#[async_trait]
pub trait ExchangeConnector: Send + Sync {
    /// 连接器名称
    fn name(&self) -> &str;
    
    /// 支持的交易对
    fn supported_symbols(&self) -> &[String];
    
    /// 连接到交易所
    async fn connect(&mut self) -> Result<()>;
    
    /// 断开连接
    async fn disconnect(&mut self) -> Result<()>;
    
    /// 订阅市场数据
    async fn subscribe(&mut self, symbols: &[String], data_types: &[String]) -> Result<()>;
    
    /// 取消订阅
    async fn unsubscribe(&mut self, symbols: &[String], data_types: &[String]) -> Result<()>;
    
    /// 检查连接状态
    fn is_connected(&self) -> bool;
    
    /// 获取连接统计
    fn get_stats(&self) -> ConnectionStats;
    
    /// 处理原始消息
    async fn handle_message(&mut self, message: &str) -> Result<Vec<MarketDataEvent>>;
}

/// 市场数据事件
#[derive(Debug, Clone)]
pub enum MarketDataEvent {
    Tick(MarketTick),
    Kline(Kline),
    OrderBook(OrderBook),
    Trade(Trade),
    Heartbeat {
        exchange: String,
        timestamp: i64,
    },
    Error {
        exchange: String,
        error: String,
        timestamp: i64,
    },
    ConnectionStatus {
        exchange: String,
        connected: bool,
        timestamp: i64,
    },
}

impl MarketDataEvent {
    /// 获取事件类型
    pub fn event_type(&self) -> &'static str {
        match self {
            MarketDataEvent::Tick(_) => "tick",
            MarketDataEvent::Kline(_) => "kline",
            MarketDataEvent::OrderBook(_) => "orderbook",
            MarketDataEvent::Trade(_) => "trade",
            MarketDataEvent::Heartbeat { .. } => "heartbeat",
            MarketDataEvent::Error { .. } => "error",
            MarketDataEvent::ConnectionStatus { .. } => "connection_status",
        }
    }

    /// 获取交易所名称
    pub fn exchange(&self) -> &str {
        match self {
            MarketDataEvent::Tick(tick) => tick.exchange.as_str(),
            MarketDataEvent::Kline(kline) => &kline.exchange,
            MarketDataEvent::OrderBook(book) => &book.exchange,
            MarketDataEvent::Trade(trade) => &trade.exchange,
            MarketDataEvent::Heartbeat { exchange, .. } => exchange,
            MarketDataEvent::Error { exchange, .. } => exchange,
            MarketDataEvent::ConnectionStatus { exchange, .. } => exchange,
        }
    }

    /// 获取时间戳
    pub fn timestamp(&self) -> i64 {
        match self {
            MarketDataEvent::Tick(tick) => tick.timestamp.timestamp(),
            MarketDataEvent::Kline(kline) => kline.open_time,
            MarketDataEvent::OrderBook(book) => book.timestamp,
            MarketDataEvent::Trade(trade) => trade.timestamp,
            MarketDataEvent::Heartbeat { timestamp, .. } => *timestamp,
            MarketDataEvent::Error { timestamp, .. } => *timestamp,
            MarketDataEvent::ConnectionStatus { timestamp, .. } => *timestamp,
        }
    }
}

/// 连接统计信息
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ConnectionStats {
    pub connected: bool,
    pub connection_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_message_time: Option<chrono::DateTime<chrono::Utc>>,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub errors_count: u64,
    pub reconnect_count: u32,
    pub subscriptions: HashMap<String, Vec<String>>, // symbol -> data_types
    pub latency_ms: Option<f64>,
}

impl ConnectionStats {
    /// 记录接收消息
    pub fn record_message_received(&mut self) {
        self.messages_received += 1;
        self.last_message_time = Some(chrono::Utc::now());
    }

    /// 记录发送消息
    pub fn record_message_sent(&mut self) {
        self.messages_sent += 1;
    }

    /// 记录错误
    pub fn record_error(&mut self) {
        self.errors_count += 1;
    }

    /// 记录重连
    pub fn record_reconnect(&mut self) {
        self.reconnect_count += 1;
    }

    /// 设置连接状态
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        if connected {
            self.connection_time = Some(chrono::Utc::now());
        }
    }

    /// 更新延迟
    pub fn update_latency(&mut self, latency_ms: f64) {
        self.latency_ms = Some(latency_ms);
    }

    /// 添加订阅
    pub fn add_subscription(&mut self, symbol: String, data_type: String) {
        self.subscriptions
            .entry(symbol)
            .or_insert_with(Vec::new)
            .push(data_type);
    }

    /// 移除订阅
    pub fn remove_subscription(&mut self, symbol: &str, data_type: &str) {
        if let Some(types) = self.subscriptions.get_mut(symbol) {
            types.retain(|t| t != data_type);
            if types.is_empty() {
                self.subscriptions.remove(symbol);
            }
        }
    }

    /// 获取连接时长
    pub fn connection_duration(&self) -> Option<chrono::Duration> {
        self.connection_time.map(|start| chrono::Utc::now() - start)
    }

    /// 获取消息速率 (messages/second)
    pub fn message_rate(&self) -> f64 {
        if let Some(duration) = self.connection_duration() {
            let seconds = duration.num_seconds() as f64;
            if seconds > 0.0 {
                return self.messages_received as f64 / seconds;
            }
        }
        0.0
    }

    /// 获取错误率
    pub fn error_rate(&self) -> f64 {
        if self.messages_received > 0 {
            self.errors_count as f64 / self.messages_received as f64 * 100.0
        } else {
            0.0
        }
    }
}

/// 订阅配置
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    pub symbol: String,
    pub data_types: Vec<String>,
    pub params: HashMap<String, Value>,
}

impl SubscriptionConfig {
    /// 创建Tick数据订阅
    pub fn ticker(symbol: String) -> Self {
        Self {
            symbol,
            data_types: vec!["ticker".to_string()],
            params: HashMap::new(),
        }
    }

    /// 创建K线数据订阅
    pub fn kline(symbol: String, interval: String) -> Self {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), Value::String(interval));
        
        Self {
            symbol,
            data_types: vec!["kline".to_string()],
            params,
        }
    }

    /// 创建深度数据订阅
    pub fn depth(symbol: String, levels: Option<u32>) -> Self {
        let mut params = HashMap::new();
        if let Some(levels) = levels {
            params.insert("levels".to_string(), Value::Number(levels.into()));
        }
        
        Self {
            symbol,
            data_types: vec!["depth".to_string()],
            params,
        }
    }

    /// 创建交易数据订阅
    pub fn trade(symbol: String) -> Self {
        Self {
            symbol,
            data_types: vec!["trade".to_string()],
            params: HashMap::new(),
        }
    }

    /// 创建组合订阅
    pub fn combined(symbol: String, data_types: Vec<String>) -> Self {
        Self {
            symbol,
            data_types,
            params: HashMap::new(),
        }
    }
}

/// 连接器错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),
    
    #[error("Message parsing failed: {0}")]
    MessageParsingFailed(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_stats() {
        let mut stats = ConnectionStats::default();
        
        // 测试消息记录
        stats.record_message_received();
        assert_eq!(stats.messages_received, 1);
        assert!(stats.last_message_time.is_some());
        
        stats.record_message_sent();
        assert_eq!(stats.messages_sent, 1);
        
        stats.record_error();
        assert_eq!(stats.errors_count, 1);
        
        // 测试订阅管理
        stats.add_subscription("BTCUSDT".to_string(), "ticker".to_string());
        assert!(stats.subscriptions.contains_key("BTCUSDT"));
        
        stats.remove_subscription("BTCUSDT", "ticker");
        assert!(!stats.subscriptions.contains_key("BTCUSDT"));
    }

    #[test]
    fn test_subscription_config() {
        let ticker_config = SubscriptionConfig::ticker("BTCUSDT".to_string());
        assert_eq!(ticker_config.symbol, "BTCUSDT");
        assert_eq!(ticker_config.data_types, vec!["ticker"]);
        
        let kline_config = SubscriptionConfig::kline("ETHUSDT".to_string(), "1m".to_string());
        assert_eq!(kline_config.symbol, "ETHUSDT");
        assert_eq!(kline_config.data_types, vec!["kline"]);
        assert!(kline_config.params.contains_key("interval"));
    }

    #[test]
    fn test_market_data_event() {
        use shared_models::market::MarketTick;
        use rust_decimal::Decimal;
        
        let tick = MarketTick {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640995200000,
            price: Decimal::new(50000, 0),
            volume: Decimal::new(100, 0),
            bid: Decimal::new(49999, 0),
            ask: Decimal::new(50001, 0),
        };
        
        let event = MarketDataEvent::Tick(tick);
        assert_eq!(event.event_type(), "tick");
        assert_eq!(event.exchange(), "binance");
        assert_eq!(event.timestamp(), 1640995200000);
    }
}