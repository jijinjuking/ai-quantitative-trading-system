use anyhow::Result;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use shared_models::market::{MarketTick, Kline, OrderBook, Trade, OrderBookLevel};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use super::{ExchangeConnector, MarketDataEvent, ConnectionStats, ConnectorError};
use crate::config::ExchangeConfig;

/// 币安WebSocket连接器
pub struct BinanceConnector {
    config: ExchangeConfig,
    stats: Arc<RwLock<ConnectionStats>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    is_connected: Arc<RwLock<bool>>,
}

impl BinanceConnector {
    /// 创建新的币安连接器
    pub fn new(config: ExchangeConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            is_connected: Arc::new(RwLock::new(false)),
        }
    }

    /// 构建WebSocket URL
    fn build_websocket_url(&self, streams: &[String]) -> String {
        if streams.is_empty() {
            "wss://stream.binance.com:9443/ws".to_string()
        } else {
            let stream_names = streams.join("/");
            format!("wss://stream.binance.com:9443/stream?streams={}", stream_names)
        }
    }

    /// 生成专业量化交易K线数据流名称
    /// 🚀 专业量化系统 - 只使用WebSocket实时数据流
    /// 🚫 绝对禁止HTTP API - 有频率限制且延迟高
    fn generate_stream_names(&self) -> Vec<String> {
        let mut streams = Vec::new();
        
        for symbol in &self.config.symbols {
            let symbol_lower = symbol.to_lowercase();
            
            // 🔥 核心K线数据流 - 多时间周期实时数据
            streams.push(format!("{}@kline_1m", symbol_lower));   // 1分钟K线
            streams.push(format!("{}@kline_5m", symbol_lower));   // 5分钟K线
            streams.push(format!("{}@kline_15m", symbol_lower));  // 15分钟K线
            streams.push(format!("{}@kline_1h", symbol_lower));   // 1小时K线
            streams.push(format!("{}@kline_4h", symbol_lower));   // 4小时K线
            streams.push(format!("{}@kline_1d", symbol_lower));   // 日K线
            
            // 📊 实时市场数据流
            streams.push(format!("{}@ticker", symbol_lower));     // 24小时统计
            streams.push(format!("{}@bookTicker", symbol_lower)); // 最佳买卖价
            streams.push(format!("{}@trade", symbol_lower));      // 实时成交
            
            // 📈 深度数据流 (高频交易必需)
            streams.push(format!("{}@depth20@100ms", symbol_lower)); // 20档深度100ms推送
        }
        
        info!("🚀 生成{}个交易对的WebSocket数据流，共{}个流", 
              self.config.symbols.len(), streams.len());
        streams
    }

    /// 解析WebSocket消息
    async fn parse_message(&self, message: &str) -> Result<Vec<MarketDataEvent>> {
        let mut events = Vec::new();
        
        // 尝试解析为流数据格式
        if let Ok(stream_data) = serde_json::from_str::<BinanceStreamData>(message) {
            match stream_data.data {
                BinanceData::Ticker(ticker_data) => {
                    if let Ok(tick) = self.parse_ticker(&ticker_data).await {
                        events.push(MarketDataEvent::Tick(tick));
                    }
                }
                BinanceData::Kline(kline_data) => {
                    if let Ok(kline) = self.parse_kline(&kline_data).await {
                        events.push(MarketDataEvent::Kline(kline));
                    }
                }
                BinanceData::BookTicker(book_data) => {
                    if let Ok(orderbook) = self.parse_book_ticker(&book_data).await {
                        events.push(MarketDataEvent::OrderBook(orderbook));
                    }
                }
                BinanceData::Trade(trade_data) => {
                    if let Ok(trade) = self.parse_trade(&trade_data).await {
                        events.push(MarketDataEvent::Trade(trade));
                    }
                }
            }
        } else {
            // 尝试直接解析各种数据格式
            if let Ok(ticker_data) = serde_json::from_str::<BinanceTickerData>(message) {
                if let Ok(tick) = self.parse_ticker(&ticker_data).await {
                    events.push(MarketDataEvent::Tick(tick));
                }
            }
        }
        
        Ok(events)
    }

    /// 解析Ticker数据
    async fn parse_ticker(&self, data: &BinanceTickerData) -> Result<MarketTick> {
        Ok(MarketTick {
            exchange: "binance".to_string(),
            symbol: data.s.clone(),
            timestamp: data.E,
            price: data.c.parse()?,
            volume: data.v.parse()?,
            bid: data.b.parse()?,
            ask: data.a.parse()?,
        })
    }

    /// 解析K线数据
    async fn parse_kline(&self, data: &BinanceKlineData) -> Result<Kline> {
        let k = &data.k;
        Ok(Kline {
            exchange: "binance".to_string(),
            symbol: k.s.clone(),
            interval: k.i.clone(),
            open_time: k.t,
            close_time: k.T,
            open: k.o.parse()?,
            high: k.h.parse()?,
            low: k.l.parse()?,
            close: k.c.parse()?,
            volume: k.v.parse()?,
            quote_volume: k.q.parse()?,
            trade_count: k.n,
            taker_buy_volume: k.V.parse()?,
            taker_buy_quote_volume: k.Q.parse()?,
            is_closed: k.x,
            is_backfilled: false,
        })
    }

    /// 解析BookTicker数据
    async fn parse_book_ticker(&self, data: &BinanceBookTickerData) -> Result<OrderBook> {
        Ok(OrderBook {
            exchange: "binance".to_string(),
            symbol: data.s.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            bids: vec![OrderBookLevel {
                price: data.b.parse()?,
                quantity: data.B.parse()?,
            }],
            asks: vec![OrderBookLevel {
                price: data.a.parse()?,
                quantity: data.A.parse()?,
            }],
        })
    }

    /// 解析交易数据
    async fn parse_trade(&self, data: &BinanceTradeData) -> Result<Trade> {
        Ok(Trade {
            exchange: "binance".to_string(),
            symbol: data.s.clone(),
            timestamp: data.T,
            trade_id: data.t.to_string(),
            price: data.p.parse()?,
            quantity: data.q.parse()?,
            side: if data.m { "sell".to_string() } else { "buy".to_string() },
            is_buyer_maker: data.m,
            is_backfilled: false,
        })
    }
}

#[async_trait]
impl ExchangeConnector for BinanceConnector {
    fn name(&self) -> &str {
        "binance"
    }

    fn supported_symbols(&self) -> &[String] {
        &self.config.symbols
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Binance WebSocket...");
        
        let streams = self.generate_stream_names();
        let url = self.build_websocket_url(&streams);
        
        let url = Url::parse(&url)?;
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| ConnectorError::ConnectionFailed(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();
        
        // 更新连接状态
        *self.is_connected.write().await = true;
        self.stats.write().await.set_connected(true);
        
        info!("Connected to Binance WebSocket");

        // 启动消息处理循环
        let stats = self.stats.clone();
        let is_connected = self.is_connected.clone();
        
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        stats.write().await.record_message_received();
                        
                        // 这里应该将消息发送到数据处理器
                        // 由于架构限制，这里只是记录日志
                        debug!("Received message: {}", text);
                    }
                    Ok(Message::Ping(ping)) => {
                        // 响应ping
                        if let Err(e) = write.send(Message::Pong(ping)).await {
                            error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        stats.write().await.record_error();
                        break;
                    }
                    _ => {}
                }
            }
            
            // 连接断开
            *is_connected.write().await = false;
            stats.write().await.set_connected(false);
            warn!("Binance WebSocket connection lost");
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from Binance WebSocket...");
        
        *self.is_connected.write().await = false;
        self.stats.write().await.set_connected(false);
        
        info!("Disconnected from Binance WebSocket");
        Ok(())
    }

    async fn subscribe(&mut self, symbols: &[String], data_types: &[String]) -> Result<()> {
        info!("Subscribing to {} symbols with {} data types", symbols.len(), data_types.len());
        
        let mut subscriptions = self.subscriptions.write().await;
        for symbol in symbols {
            subscriptions.insert(symbol.clone(), data_types.to_vec());
            
            let mut stats = self.stats.write().await;
            for data_type in data_types {
                stats.add_subscription(symbol.clone(), data_type.clone());
            }
        }
        
        Ok(())
    }

    async fn unsubscribe(&mut self, symbols: &[String], data_types: &[String]) -> Result<()> {
        info!("Unsubscribing from {} symbols", symbols.len());
        
        let mut subscriptions = self.subscriptions.write().await;
        for symbol in symbols {
            subscriptions.remove(symbol);
            
            let mut stats = self.stats.write().await;
            for data_type in data_types {
                stats.remove_subscription(symbol, data_type);
            }
        }
        
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // 这里需要同步访问，在实际实现中可能需要调整
        false
    }

    fn get_stats(&self) -> ConnectionStats {
        // 这里需要同步访问，在实际实现中可能需要调整
        ConnectionStats::default()
    }

    async fn handle_message(&mut self, message: &str) -> Result<Vec<MarketDataEvent>> {
        self.parse_message(message).await
    }
}

/// 币安流数据格式
#[derive(Debug, Deserialize)]
struct BinanceStreamData {
    stream: String,
    data: BinanceData,
}

/// 币安数据类型
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BinanceData {
    Ticker(BinanceTickerData),
    Kline(BinanceKlineData),
    BookTicker(BinanceBookTickerData),
    Trade(BinanceTradeData),
}

/// 币安Ticker数据
#[derive(Debug, Deserialize)]
struct BinanceTickerData {
    #[serde(rename = "E")]
    E: i64,  // 事件时间
    #[serde(rename = "s")]
    s: String,  // 交易对
    #[serde(rename = "c")]
    c: String,  // 最新价格
    #[serde(rename = "v")]
    v: String,  // 24小时成交量
    #[serde(rename = "b")]
    b: String,  // 最佳买价
    #[serde(rename = "a")]
    a: String,  // 最佳卖价
}

/// 币安K线数据
#[derive(Debug, Deserialize)]
struct BinanceKlineData {
    #[serde(rename = "E")]
    E: i64,  // 事件时间
    #[serde(rename = "s")]
    s: String,  // 交易对
    #[serde(rename = "k")]
    k: BinanceKlineInfo,
}

#[derive(Debug, Deserialize)]
struct BinanceKlineInfo {
    #[serde(rename = "t")]
    t: i64,     // K线开始时间
    #[serde(rename = "T")]
    T: i64,     // K线结束时间
    #[serde(rename = "s")]
    s: String,  // 交易对
    #[serde(rename = "i")]
    i: String,  // 时间间隔
    #[serde(rename = "o")]
    o: String,  // 开盘价
    #[serde(rename = "c")]
    c: String,  // 收盘价
    #[serde(rename = "h")]
    h: String,  // 最高价
    #[serde(rename = "l")]
    l: String,  // 最低价
    #[serde(rename = "v")]
    v: String,  // 成交量
    #[serde(rename = "n")]
    n: u64,     // 成交笔数
    #[serde(rename = "x")]
    x: bool,    // 是否完结
    #[serde(rename = "q")]
    q: String,  // 成交额
    #[serde(rename = "V")]
    V: String,  // 主动买入成交量
    #[serde(rename = "Q")]
    Q: String,  // 主动买入成交额
}

/// 币安BookTicker数据
#[derive(Debug, Deserialize)]
struct BinanceBookTickerData {
    #[serde(rename = "s")]
    s: String,  // 交易对
    #[serde(rename = "b")]
    b: String,  // 最佳买价
    #[serde(rename = "B")]
    B: String,  // 最佳买量
    #[serde(rename = "a")]
    a: String,  // 最佳卖价
    #[serde(rename = "A")]
    A: String,  // 最佳卖量
}

/// 币安交易数据
#[derive(Debug, Deserialize)]
struct BinanceTradeData {
    #[serde(rename = "E")]
    E: i64,     // 事件时间
    #[serde(rename = "s")]
    s: String,  // 交易对
    #[serde(rename = "t")]
    t: u64,     // 交易ID
    #[serde(rename = "p")]
    p: String,  // 价格
    #[serde(rename = "q")]
    q: String,  // 数量
    #[serde(rename = "T")]
    T: i64,     // 交易时间
    #[serde(rename = "m")]
    m: bool,    // 是否为做市方
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_name_generation() {
        let config = ExchangeConfig {
            enabled: true,
            symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            ..Default::default()
        };
        
        let connector = BinanceConnector::new(config);
        let streams = connector.generate_stream_names();
        
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@kline_1m".to_string()));
        assert!(streams.contains(&"ethusdt@ticker".to_string()));
    }

    #[tokio::test]
    async fn test_ticker_parsing() {
        let connector = BinanceConnector::new(ExchangeConfig::default());
        
        let ticker_data = BinanceTickerData {
            E: 1640995200000,
            s: "BTCUSDT".to_string(),
            c: "50000.00".to_string(),
            v: "1000.00".to_string(),
            b: "49999.00".to_string(),
            a: "50001.00".to_string(),
        };
        
        let tick = connector.parse_ticker(&ticker_data).await.unwrap();
        assert_eq!(tick.symbol, "BTCUSDT");
        assert_eq!(tick.exchange, "binance");
    }
}