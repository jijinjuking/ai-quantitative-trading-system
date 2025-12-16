pub mod server;
pub mod connection;
pub mod message;
pub mod subscription;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use serde::{Deserialize, Serialize};
use shared_models::market::{MarketTick, Kline, OrderBook, Trade};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

pub use server::WebSocketServer;
pub use connection::{WebSocketConnection, ConnectionManager};
pub use message::{WebSocketMessage, MessageType, SubscriptionRequest, SubscriptionResponse};
pub use subscription::{SubscriptionManager, Subscription, SubscriptionFilter};

use crate::config::MarketDataConfig;
use crate::processors::DataEvent;

/// WebSocket事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketEvent {
    /// 市场Tick数据
    Tick(MarketTick),
    /// K线数据
    Kline(Kline),
    /// 订单簿数据
    OrderBook(OrderBook),
    /// 交易数据
    Trade(Trade),
    /// 连接状态变化
    ConnectionStatus {
        exchange: String,
        connected: bool,
        timestamp: i64,
    },
    /// 错误信息
    Error {
        code: u32,
        message: String,
        timestamp: i64,
    },
    /// 心跳
    Heartbeat {
        timestamp: i64,
    },
}

impl WebSocketEvent {
    /// 从数据事件转换
    pub fn from_data_event(event: &DataEvent) -> Option<Self> {
        match event {
            DataEvent::Tick(tick) => Some(WebSocketEvent::Tick(tick.clone())),
            DataEvent::Kline(kline) => Some(WebSocketEvent::Kline(kline.clone())),
            DataEvent::OrderBook(book) => Some(WebSocketEvent::OrderBook(book.clone())),
            DataEvent::Trade(trade) => Some(WebSocketEvent::Trade(trade.clone())),
            DataEvent::ConnectionStatus { exchange, connected } => {
                Some(WebSocketEvent::ConnectionStatus {
                    exchange: exchange.clone(),
                    connected: *connected,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                })
            }
            DataEvent::Error { exchange, error } => {
                Some(WebSocketEvent::Error {
                    code: 500,
                    message: format!("{}: {}", exchange, error),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                })
            }
        }
    }

    /// 获取事件类型字符串
    pub fn event_type(&self) -> &'static str {
        match self {
            WebSocketEvent::Tick(_) => "tick",
            WebSocketEvent::Kline(_) => "kline",
            WebSocketEvent::OrderBook(_) => "orderbook",
            WebSocketEvent::Trade(_) => "trade",
            WebSocketEvent::ConnectionStatus { .. } => "connection_status",
            WebSocketEvent::Error { .. } => "error",
            WebSocketEvent::Heartbeat { .. } => "heartbeat",
        }
    }

    /// 获取交易所名称
    pub fn exchange(&self) -> Option<&str> {
        match self {
            WebSocketEvent::Tick(tick) => Some(tick.exchange.as_str()),
            WebSocketEvent::Kline(kline) => Some(kline.exchange.as_str()),
            WebSocketEvent::OrderBook(book) => Some(book.exchange.as_str()),
            WebSocketEvent::Trade(trade) => Some(trade.exchange.as_str()),
            WebSocketEvent::ConnectionStatus { exchange, .. } => Some(exchange),
            _ => None,
        }
    }

    /// 获取交易对
    pub fn symbol(&self) -> Option<&str> {
        match self {
            WebSocketEvent::Tick(tick) => Some(&tick.symbol),
            WebSocketEvent::Kline(kline) => Some(&kline.symbol),
            WebSocketEvent::OrderBook(book) => Some(&book.symbol),
            WebSocketEvent::Trade(trade) => Some(&trade.symbol),
            _ => None,
        }
    }

    /// 序列化为JSON字符串
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize WebSocket event: {}", e))
    }

    /// 从JSON字符串反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize WebSocket event: {}", e))
    }
}

/// WebSocket错误类型
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Message parsing failed: {0}")]
    MessageParsingFailed(String),
    
    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl WebSocketError {
    /// 转换为WebSocket事件
    pub fn to_event(&self) -> WebSocketEvent {
        let (code, message) = match self {
            WebSocketError::ConnectionFailed(msg) => (1001, msg.clone()),
            WebSocketError::MessageParsingFailed(msg) => (1002, msg.clone()),
            WebSocketError::SubscriptionFailed(msg) => (1003, msg.clone()),
            WebSocketError::RateLimitExceeded(msg) => (1004, msg.clone()),
            WebSocketError::AuthenticationFailed(msg) => (1005, msg.clone()),
            WebSocketError::InvalidRequest(msg) => (1006, msg.clone()),
            WebSocketError::InternalError(msg) => (1007, msg.clone()),
        };

        WebSocketEvent::Error {
            code,
            message,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// WebSocket统计信息
#[derive(Debug, Clone, Default, Serialize)]
pub struct WebSocketStats {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_subscriptions: u64,
    pub active_subscriptions: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors_count: u64,
    pub average_message_size: f64,
    pub messages_per_second: f64,
    pub connection_duration_avg_seconds: f64,
}

impl WebSocketStats {
    /// 记录新连接
    pub fn record_connection(&mut self) {
        self.total_connections += 1;
        self.active_connections += 1;
    }

    /// 记录连接断开
    pub fn record_disconnection(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    /// 记录发送消息
    pub fn record_message_sent(&mut self, bytes: u64) {
        self.total_messages_sent += 1;
        self.bytes_sent += bytes;
        self.update_average_message_size();
    }

    /// 记录接收消息
    pub fn record_message_received(&mut self, bytes: u64) {
        self.total_messages_received += 1;
        self.bytes_received += bytes;
        self.update_average_message_size();
    }

    /// 记录订阅
    pub fn record_subscription(&mut self) {
        self.total_subscriptions += 1;
        self.active_subscriptions += 1;
    }

    /// 记录取消订阅
    pub fn record_unsubscription(&mut self) {
        if self.active_subscriptions > 0 {
            self.active_subscriptions -= 1;
        }
    }

    /// 记录错误
    pub fn record_error(&mut self) {
        self.errors_count += 1;
    }

    /// 更新平均消息大小
    fn update_average_message_size(&mut self) {
        let total_messages = self.total_messages_sent + self.total_messages_received;
        let total_bytes = self.bytes_sent + self.bytes_received;
        
        if total_messages > 0 {
            self.average_message_size = total_bytes as f64 / total_messages as f64;
        }
    }

    /// 计算错误率
    pub fn error_rate(&self) -> f64 {
        let total_messages = self.total_messages_sent + self.total_messages_received;
        if total_messages > 0 {
            self.errors_count as f64 / total_messages as f64 * 100.0
        } else {
            0.0
        }
    }
}

/// WebSocket配置
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub max_connections: usize,
    pub max_message_size: usize,
    pub heartbeat_interval: std::time::Duration,
    pub connection_timeout: std::time::Duration,
    pub rate_limit_messages_per_second: u32,
    pub rate_limit_burst_size: u32,
    pub enable_compression: bool,
    pub buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 10000,
            max_message_size: 1024 * 1024, // 1MB
            heartbeat_interval: std::time::Duration::from_secs(30),
            connection_timeout: std::time::Duration::from_secs(60),
            rate_limit_messages_per_second: 100,
            rate_limit_burst_size: 200,
            enable_compression: true,
            buffer_size: 1000,
        }
    }
}

/// WebSocket广播器
pub struct WebSocketBroadcaster {
    sender: broadcast::Sender<WebSocketEvent>,
    stats: Arc<RwLock<WebSocketStats>>,
}

impl WebSocketBroadcaster {
    /// 创建新的广播器
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer_size);
        
        Self {
            sender,
            stats: Arc::new(RwLock::new(WebSocketStats::default())),
        }
    }

    /// 广播事件
    pub async fn broadcast(&self, event: WebSocketEvent) -> Result<()> {
        let json = event.to_json()?;
        let bytes = json.len() as u64;

        match self.sender.send(event) {
            Ok(receiver_count) => {
                let mut stats = self.stats.write().await;
                stats.record_message_sent(bytes * receiver_count as u64);
                
                debug!("Broadcasted event to {} receivers", receiver_count);
                Ok(())
            }
            Err(_) => {
                // 没有接收者，这是正常的
                debug!("No receivers for broadcast event");
                Ok(())
            }
        }
    }

    /// 订阅事件流
    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketEvent> {
        self.sender.subscribe()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> WebSocketStats {
        self.stats.read().await.clone()
    }

    /// 获取接收者数量
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// WebSocket事件过滤器
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub exchanges: Option<Vec<String>>,
    pub symbols: Option<Vec<String>>,
    pub event_types: Option<Vec<String>>,
}

impl EventFilter {
    /// 创建空过滤器（允许所有事件）
    pub fn allow_all() -> Self {
        Self {
            exchanges: None,
            symbols: None,
            event_types: None,
        }
    }

    /// 创建交易所过滤器
    pub fn exchanges(exchanges: Vec<String>) -> Self {
        Self {
            exchanges: Some(exchanges),
            symbols: None,
            event_types: None,
        }
    }

    /// 创建交易对过滤器
    pub fn symbols(symbols: Vec<String>) -> Self {
        Self {
            exchanges: None,
            symbols: Some(symbols),
            event_types: None,
        }
    }

    /// 创建事件类型过滤器
    pub fn event_types(event_types: Vec<String>) -> Self {
        Self {
            exchanges: None,
            symbols: None,
            event_types: Some(event_types),
        }
    }

    /// 检查事件是否匹配过滤器
    pub fn matches(&self, event: &WebSocketEvent) -> bool {
        // 检查交易所过滤器
        if let Some(exchanges) = &self.exchanges {
            if let Some(exchange) = event.exchange() {
                if !exchanges.contains(&exchange.to_string()) {
                    return false;
                }
            }
        }

        // 检查交易对过滤器
        if let Some(symbols) = &self.symbols {
            if let Some(symbol) = event.symbol() {
                if !symbols.contains(&symbol.to_string()) {
                    return false;
                }
            }
        }

        // 检查事件类型过滤器
        if let Some(event_types) = &self.event_types {
            if !event_types.contains(&event.event_type().to_string()) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_websocket_event_serialization() {
        let tick = MarketTick {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640995200000,
            price: Decimal::new(50000, 0),
            volume: Decimal::new(100, 0),
            bid: Decimal::new(49999, 0),
            ask: Decimal::new(50001, 0),
        };

        let event = WebSocketEvent::Tick(tick);
        let json = event.to_json().unwrap();
        let deserialized = WebSocketEvent::from_json(&json).unwrap();

        match deserialized {
            WebSocketEvent::Tick(deserialized_tick) => {
                assert_eq!(deserialized_tick.exchange, "binance");
                assert_eq!(deserialized_tick.symbol, "BTCUSDT");
            }
            _ => panic!("Expected Tick event"),
        }
    }

    #[test]
    fn test_event_filter() {
        let tick = MarketTick {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640995200000,
            price: Decimal::new(50000, 0),
            volume: Decimal::new(100, 0),
            bid: Decimal::new(49999, 0),
            ask: Decimal::new(50001, 0),
        };

        let event = WebSocketEvent::Tick(tick);

        // 测试交易所过滤器
        let exchange_filter = EventFilter::exchanges(vec!["binance".to_string()]);
        assert!(exchange_filter.matches(&event));

        let wrong_exchange_filter = EventFilter::exchanges(vec!["okx".to_string()]);
        assert!(!wrong_exchange_filter.matches(&event));

        // 测试交易对过滤器
        let symbol_filter = EventFilter::symbols(vec!["BTCUSDT".to_string()]);
        assert!(symbol_filter.matches(&event));

        let wrong_symbol_filter = EventFilter::symbols(vec!["ETHUSDT".to_string()]);
        assert!(!wrong_symbol_filter.matches(&event));

        // 测试事件类型过滤器
        let type_filter = EventFilter::event_types(vec!["tick".to_string()]);
        assert!(type_filter.matches(&event));

        let wrong_type_filter = EventFilter::event_types(vec!["kline".to_string()]);
        assert!(!wrong_type_filter.matches(&event));
    }

    #[test]
    fn test_websocket_stats() {
        let mut stats = WebSocketStats::default();
        
        stats.record_connection();
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);
        
        stats.record_message_sent(100);
        assert_eq!(stats.total_messages_sent, 1);
        assert_eq!(stats.bytes_sent, 100);
        assert_eq!(stats.average_message_size, 100.0);
        
        stats.record_disconnection();
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_websocket_broadcaster() {
        let broadcaster = WebSocketBroadcaster::new(100);
        
        let tick = MarketTick {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640995200000,
            price: Decimal::new(50000, 0),
            volume: Decimal::new(100, 0),
            bid: Decimal::new(49999, 0),
            ask: Decimal::new(50001, 0),
        };

        let event = WebSocketEvent::Tick(tick);
        
        // 没有订阅者时广播应该成功
        let result = broadcaster.broadcast(event).await;
        assert!(result.is_ok());
        
        assert_eq!(broadcaster.receiver_count(), 0);
    }
}