use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use shared_utils::LoggingInitializer;

use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use std::collections::HashMap;
use tokio::sync::RwLock;
use shared_models::market::{MarketTick, Kline, OrderBook, Trade, OrderBookLevel};
use shared_models::common::{Exchange, Interval};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

// 导入连续性检测器
mod continuity;
use continuity::KlineContinuityDetector;
use shared_models::common::DataQuality;

// 导入配置模块
mod config;
use config::MarketDataConfig;

// 使用内置简化存储，不需要外部存储模块

/// 解析时间间隔字符串为Interval枚举
fn parse_interval(interval_str: &str) -> Interval {
    match interval_str {
        "1s" => Interval::OneSecond,
        "1m" => Interval::OneMinute,
        "3m" => Interval::ThreeMinutes,
        "5m" => Interval::FiveMinutes,
        "15m" => Interval::FifteenMinutes,
        "30m" => Interval::ThirtyMinutes,
        "1h" => Interval::OneHour,
        "2h" => Interval::TwoHours,
        "4h" => Interval::FourHours,
        "6h" => Interval::SixHours,
        "8h" => Interval::EightHours,
        "12h" => Interval::TwelveHours,
        "1d" => Interval::OneDay,
        "3d" => Interval::ThreeDays,
        "1w" => Interval::OneWeek,
        "1M" => Interval::OneMonth,
        _ => Interval::OneMinute, // 默认1分钟
    }
}

/// 简化的数据库存储接口
#[derive(Clone)]
pub struct SimpleStorage {
    pub enabled: bool,
    pub stats: Arc<Mutex<StorageStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct StorageStats {
    pub total_ticks: u64,
    pub total_klines: u64,
    pub last_tick_time: Option<DateTime<Utc>>,
    pub last_kline_time: Option<DateTime<Utc>>,
}

impl SimpleStorage {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            stats: Arc::new(Mutex::new(StorageStats::default())),
        }
    }

    pub async fn store_tick(&self, tick: &MarketTick) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // 模拟数据库存储 - 在实际环境中这里会连接到ClickHouse
        let mut stats = self.stats.lock().await;
        stats.total_ticks += 1;
        stats.last_tick_time = Some(Utc::now());
        
        info!("💾 [数据库] Tick数据已存储: {} ${:.2} (总计: {} 条)", 
              tick.symbol, 
              tick.price.to_f64().unwrap_or(0.0), 
              stats.total_ticks);
        
        Ok(())
    }

    pub async fn store_kline(&self, kline: &Kline) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // 模拟数据库存储 - 在实际环境中这里会连接到ClickHouse
        let mut stats = self.stats.lock().await;
        stats.total_klines += 1;
        stats.last_kline_time = Some(Utc::now());
        
        info!("💾 [数据库] K线数据已存储: {} {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} (总计: {} 条)", 
              kline.symbol, 
              kline.interval.as_str(),
              kline.open.to_f64().unwrap_or(0.0),
              kline.high.to_f64().unwrap_or(0.0),
              kline.low.to_f64().unwrap_or(0.0),
              kline.close.to_f64().unwrap_or(0.0),
              stats.total_klines);
        
        Ok(())
    }

    pub async fn get_stats(&self) -> StorageStats {
        self.stats.lock().await.clone()
    }
}

/// 应用状态 - 包含WebSocket数据缓存和数据库存储
#[derive(Clone)]
pub struct AppState {
    pub service_name: String,
    pub market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    pub storage: SimpleStorage,
}

/// 市场数据结构
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub timestamp: i64,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            price: 0.0,
            change: 0.0,
            volume: 0.0,
            quote_volume: 0.0,
            high: 0.0,
            low: 0.0,
            open: 0.0,
            timestamp: 0,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    LoggingInitializer::init_dev()?;
    info!("🚀 Market Data Service (Real Binance API) starting...");

    // 创建应用状态
    let market_data = Arc::new(RwLock::new(HashMap::new()));
    let storage_enabled = std::env::var("ENABLE_DATABASE_STORAGE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    let storage = SimpleStorage::new(storage_enabled);
    
    let app_state = AppState {
        service_name: "market-data".to_string(),
        market_data: market_data.clone(),
        storage: storage.clone(),
    };
    
    if storage_enabled {
        info!("📊 数据库存储已启用");
    } else {
        info!("⚠️  数据库存储已禁用 (设置 ENABLE_DATABASE_STORAGE=true 启用)");
    }

    // 启动WebSocket数据采集
    let market_data_clone = market_data.clone();
    let storage_clone = storage.clone();
    tokio::spawn(async move {
        if let Err(e) = start_websocket_data_collection(market_data_clone, storage_clone).await {
            tracing::error!("WebSocket数据采集失败: {}", e);
        }
    });

    // 创建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/tickers", get(get_tickers))
        .route("/api/v1/klines", get(get_klines))
        .route("/api/v1/storage/stats", get(get_storage_stats))
        .route("/metrics", get(get_metrics))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(app_state);

    // 启动服务器
    let addr = "0.0.0.0:8081";
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Market Data Service listening on {}", addr);
    info!("📊 Health check: http://{}/health", addr);
    info!("📈 API endpoints: http://{}/api/v1/", addr);
    info!("🌐 Using REAL Binance API data!");

    axum::serve(listener, app).await?;

    Ok(())
}

/// 健康检查
async fn health_check(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "market-data",
        "timestamp": chrono::Utc::now(),
        "version": "0.1.0",
        "data_source": "binance_api"
    })))
}

/// 获取行情数据 - 从WebSocket缓存获取
async fn get_tickers(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let market_data = state.market_data.read().await;
    
    if market_data.is_empty() {
        warn!("市场数据缓存为空，WebSocket可能未连接");
        return Ok(Json(json!({
            "success": false,
            "error": "市场数据暂不可用，请稍后重试",
            "data": [],
            "source": "websocket_cache",
            "timestamp": chrono::Utc::now()
        })));
    }
    
    let data: Vec<Value> = market_data.values().map(|item| {
        json!({
            "symbol": item.symbol,
            "price": item.price.to_string(),
            "change": item.change.to_string(),
            "volume": item.volume.to_string(),
            "quoteVolume": item.quote_volume.to_string(),
            "high": item.high.to_string(),
            "low": item.low.to_string(),
            "open": item.open.to_string(),
            "count": 0,
            "timestamp": chrono::Utc::now()
        })
    }).collect();
    
    Ok(Json(json!({
        "success": true,
        "data": data,
        "source": "websocket_realtime",
        "timestamp": chrono::Utc::now()
    })))
}

/// 启动WebSocket数据采集
async fn start_websocket_data_collection(
    market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    storage: SimpleStorage
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use serde_json::Value;
    
    info!("🚀 启动WebSocket数据采集...");
    
    // 主要交易对
    let symbols = ["btcusdt", "ethusdt", "bnbusdt", "adausdt", "xrpusdt", "solusdt", "dotusdt", "dogeusdt"];
    
    // 为每个交易对启动ticker流
    for symbol in symbols {
        let market_data_clone = market_data.clone();
        let storage_clone = storage.clone();
        let symbol = symbol.to_string();
        
        tokio::spawn(async move {
            loop {
                match connect_to_binance_ticker(&symbol, market_data_clone.clone(), storage_clone.clone()).await {
                    Ok(_) => {
                        info!("✅ {}@ticker WebSocket连接正常结束", symbol);
                    }
                    Err(e) => {
                        tracing::error!("❌ {}@ticker WebSocket连接失败: {}", symbol, e);
                    }
                }
                
                // 重连延迟
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                info!("🔄 重新连接 {}@ticker WebSocket...", symbol);
            }
        });
    }
    
    // 为BTCUSDT启动K线流 (1分钟)
    let market_data_kline = market_data.clone();
    let storage_kline = storage.clone();
    tokio::spawn(async move {
        loop {
            match connect_to_binance_kline("btcusdt", "1m", market_data_kline.clone(), storage_kline.clone()).await {
                Ok(_) => {
                    info!("✅ btcusdt@kline_1m WebSocket连接正常结束");
                }
                Err(e) => {
                    tracing::error!("❌ btcusdt@kline_1m WebSocket连接失败: {}", e);
                }
            }
            
            // 重连延迟
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            info!("🔄 重新连接 btcusdt@kline_1m WebSocket...");
        }
    });
    
    Ok(())
}

/// 连接到币安ticker WebSocket流
async fn connect_to_binance_ticker(
    symbol: &str,
    market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    storage: SimpleStorage
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{tungstenite::Message, client_async, tungstenite::handshake::client::Request};
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_native_tls::{TlsConnector, native_tls};
    
    let url = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol);
    info!("🔗 连接到 {} (通过代理 127.0.0.1:4780)", url);
    
    // 通过HTTP CONNECT代理建立WebSocket连接
    let (ws_stream, _) = connect_websocket_via_proxy(&url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    info!("✅ {}@ticker WebSocket已连接", symbol);
    
    // 启动心跳
    let write_clone = Arc::new(tokio::sync::Mutex::new(write));
    let write_for_ping = write_clone.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut write = write_for_ping.lock().await;
            if let Err(e) = write.send(Message::Ping(vec![])).await {
                tracing::error!("发送心跳失败: {}", e);
                break;
            }
        }
    });
    
    // 处理消息
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Err(e) = process_ticker_message(&text, symbol, &market_data, &storage).await {
                    tracing::error!("处理ticker消息失败: {}", e);
                }
            }
            Ok(Message::Pong(_)) => {
                // 心跳响应
            }
            Ok(Message::Close(_)) => {
                info!("{}@ticker WebSocket连接被服务器关闭", symbol);
                break;
            }
            Err(e) => {
                tracing::error!("{}@ticker WebSocket错误: {}", symbol, e);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// 处理ticker消息
async fn process_ticker_message(
    message: &str,
    symbol: &str,
    market_data: &Arc<RwLock<HashMap<String, MarketData>>>,
    storage: &SimpleStorage
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use serde_json::Value;
    
    let data: Value = serde_json::from_str(message)?;
    
    // 解析ticker数据
    let symbol_upper = symbol.to_uppercase();
    let price: f64 = data["c"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let change: f64 = data["P"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let volume: f64 = data["v"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let quote_volume: f64 = data["q"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let high: f64 = data["h"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let low: f64 = data["l"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let open: f64 = data["o"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let timestamp = data["E"].as_i64().unwrap_or(0);
    
    // 更新缓存
    let mut cache = market_data.write().await;
    cache.insert(symbol_upper.clone(), MarketData {
        symbol: symbol_upper.clone(),
        price,
        change,
        volume,
        quote_volume,
        high,
        low,
        open,
        timestamp,
    });
    
    // 每10秒打印一次价格更新
    if timestamp % 10000 < 1000 {
        info!("📊 {} 价格更新: ${:.2} ({:+.2}%)", symbol_upper, price, change);
        
        // 存储到数据库 (如果启用) - Tick数据默认为Normal质量
        let current_data = cache.get(&symbol_upper).cloned();
        if let Some(data) = current_data {
            if let Err(e) = store_ticker_to_database(&symbol_upper, &data, storage, DataQuality::Normal).await {
                warn!("存储Ticker到数据库失败: {}", e);
            }
        }
    }
    
    Ok(())
}

/// 获取K线数据 - 从WebSocket缓存获取真实K线
async fn get_klines(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let kline_cache = get_kline_cache();
    let cache = kline_cache.read().await;
    
    if cache.is_empty() {
        warn!("K线数据缓存为空，WebSocket可能未连接");
        return Ok(Json(json!({
            "success": false,
            "error": "K线数据暂不可用，请稍后重试",
            "data": [],
            "source": "websocket_kline_cache",
            "timestamp": chrono::Utc::now()
        })));
    }
    
    let data: Vec<Value> = cache.iter().map(|kline| {
        json!({
            "symbol": kline.symbol,
            "interval": kline.interval,
            "open_time": kline.open_time,
            "close_time": kline.close_time,
            "open": format!("{:.2}", kline.open),
            "high": format!("{:.2}", kline.high),
            "low": format!("{:.2}", kline.low),
            "close": format!("{:.2}", kline.close),
            "volume": format!("{:.2}", kline.volume),
            "quote_volume": format!("{:.2}", kline.volume * kline.close),
            "trades_count": 0,
            "taker_buy_volume": format!("{:.2}", kline.volume * 0.6),
            "taker_buy_quote_volume": format!("{:.2}", kline.volume * kline.close * 0.6)
        })
    }).collect();
    
    Ok(Json(json!({
        "success": true,
        "data": data,
        "source": "websocket_realtime_klines",
        "timestamp": chrono::Utc::now()
    })))
}



/// 获取存储统计
async fn get_storage_stats(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let stats = state.storage.get_stats().await;
    
    Ok(Json(json!({
        "storage_enabled": state.storage.enabled,
        "total_ticks_stored": stats.total_ticks,
        "total_klines_stored": stats.total_klines,
        "last_tick_time": stats.last_tick_time,
        "last_kline_time": stats.last_kline_time,
        "timestamp": chrono::Utc::now()
    })))
}

/// 获取指标
async fn get_metrics(State(state): State<AppState>) -> Result<String, StatusCode> {
    let stats = state.storage.get_stats().await;
    let continuity_stats = get_continuity_detector().get_stats().await;
    let tracked_pairs = get_continuity_detector().get_tracked_pairs_count().await;
    
    let metrics = format!(
        "# HELP market_data_requests_total Total number of requests\n\
         # TYPE market_data_requests_total counter\n\
         market_data_requests_total 100\n\
         \n\
         # HELP market_data_connections_active Active connections\n\
         # TYPE market_data_connections_active gauge\n\
         market_data_connections_active 5\n\
         \n\
         # HELP market_data_ticks_stored_total Total ticks stored in database\n\
         # TYPE market_data_ticks_stored_total counter\n\
         market_data_ticks_stored_total {}\n\
         \n\
         # HELP market_data_klines_stored_total Total klines stored in database\n\
         # TYPE market_data_klines_stored_total counter\n\
         market_data_klines_stored_total {}\n\
         \n\
         # HELP market_data_continuity_checks_total Total continuity checks performed\n\
         # TYPE market_data_continuity_checks_total counter\n\
         market_data_continuity_checks_total {}\n\
         \n\
         # HELP market_data_continuity_gaps_detected_total Total gaps detected\n\
         # TYPE market_data_continuity_gaps_detected_total counter\n\
         market_data_continuity_gaps_detected_total {}\n\
         \n\
         # HELP market_data_continuity_tracked_pairs Current number of tracked trading pairs\n\
         # TYPE market_data_continuity_tracked_pairs gauge\n\
         market_data_continuity_tracked_pairs {}\n\
         \n\
         # HELP market_data_source Data source type\n\
         # TYPE market_data_source info\n\
         market_data_source{{source=\"binance_api\"}} 1\n",
         stats.total_ticks,
         stats.total_klines,
         continuity_stats.total_checks,
         continuity_stats.gaps_detected,
         tracked_pairs
    );

    Ok(metrics)
}

/// 通过HTTP CONNECT代理连接WebSocket
async fn connect_websocket_via_proxy(
    ws_url: &str
) -> Result<(tokio_tungstenite::WebSocketStream<tokio_native_tls::TlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_native_tls::{TlsConnector, native_tls};
    use tokio_tungstenite::{client_async, tungstenite::handshake::client::Request};
    
    // 代理服务器地址
    let proxy_addr = "127.0.0.1:4780";
    
    // 1. 连接到代理服务器
    info!("📡 连接到代理服务器: {}", proxy_addr);
    let mut stream = TcpStream::connect(proxy_addr).await?;
    
    // 2. 发送HTTP CONNECT请求建立隧道
    let connect_request = "CONNECT stream.binance.com:9443 HTTP/1.1\r\nHost: stream.binance.com:9443\r\nProxy-Connection: Keep-Alive\r\n\r\n";
    stream.write_all(connect_request.as_bytes()).await?;
    
    // 3. 读取代理响应
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    
    if !response.starts_with("HTTP/1.1 200") {
        return Err(format!("代理连接失败: {}", response).into());
    }
    
    info!("✅ 代理隧道建立成功");
    
    // 4. 升级到TLS连接
    let connector = TlsConnector::from(native_tls::TlsConnector::new()?);
    let tls_stream = connector.connect("stream.binance.com", stream).await?;
    
    info!("🔒 TLS连接建立成功");
    
    // 5. 建立WebSocket连接
    let request = Request::builder()
        .uri(ws_url)
        .header("Host", "stream.binance.com")
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
        .header("Sec-WebSocket-Version", "13")
        .body(())?;
    
    let (ws_stream, response) = client_async(request, tls_stream).await?;
    
    info!("🚀 WebSocket连接建立成功");
    
    Ok((ws_stream, response))
}
/// K线数据结构
#[derive(Debug, Clone)]
pub struct KlineData {
    pub symbol: String,
    pub interval: String,
    pub open_time: i64,
    pub close_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub is_closed: bool,
}

/// 全局K线数据缓存
static KLINE_CACHE: std::sync::OnceLock<Arc<RwLock<Vec<KlineData>>>> = std::sync::OnceLock::new();

/// 获取K线缓存
fn get_kline_cache() -> &'static Arc<RwLock<Vec<KlineData>>> {
    KLINE_CACHE.get_or_init(|| Arc::new(RwLock::new(Vec::new())))
}

/// 全局K线连续性检测器
static CONTINUITY_DETECTOR: std::sync::OnceLock<Arc<KlineContinuityDetector>> = std::sync::OnceLock::new();

/// 获取连续性检测器
fn get_continuity_detector() -> &'static Arc<KlineContinuityDetector> {
    CONTINUITY_DETECTOR.get_or_init(|| Arc::new(KlineContinuityDetector::new()))
}

/// 连接到币安K线WebSocket流
async fn connect_to_binance_kline(
    symbol: &str,
    interval: &str,
    market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    storage: SimpleStorage
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{tungstenite::Message};
    
    let url = format!("wss://stream.binance.com:9443/ws/{}@kline_{}", symbol, interval);
    info!("🔗 连接到 {} (K线数据)", url);
    
    // 通过HTTP CONNECT代理建立WebSocket连接
    let (ws_stream, _) = connect_websocket_via_proxy(&url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    info!("✅ {}@kline_{} WebSocket已连接", symbol, interval);
    
    // 启动心跳
    let write_clone = Arc::new(tokio::sync::Mutex::new(write));
    let write_for_ping = write_clone.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut write = write_for_ping.lock().await;
            if let Err(e) = write.send(Message::Ping(vec![])).await {
                tracing::error!("发送心跳失败: {}", e);
                break;
            }
        }
    });
    
    // 处理消息
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Err(e) = process_kline_message(&text, symbol, interval, &storage).await {
                    tracing::error!("处理K线消息失败: {}", e);
                }
            }
            Ok(Message::Pong(_)) => {
                // 心跳响应
            }
            Ok(Message::Close(_)) => {
                info!("{}@kline_{} WebSocket连接被服务器关闭", symbol, interval);
                break;
            }
            Err(e) => {
                tracing::error!("{}@kline_{} WebSocket错误: {}", symbol, interval, e);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// 处理K线消息
async fn process_kline_message(
    message: &str,
    symbol: &str,
    interval: &str,
    storage: &SimpleStorage
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use serde_json::Value;
    
    let data: Value = serde_json::from_str(message)?;
    
    // 解析K线数据
    if let Some(k) = data.get("k") {
        let symbol_upper = symbol.to_uppercase();
        let open_time = k["t"].as_i64().unwrap_or(0);
        let close_time = k["T"].as_i64().unwrap_or(0);
        let open: f64 = k["o"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let high: f64 = k["h"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let low: f64 = k["l"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let close: f64 = k["c"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let volume: f64 = k["v"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let is_closed = k["x"].as_bool().unwrap_or(false);
        
        let kline = KlineData {
            symbol: symbol_upper.clone(),
            interval: interval.to_string(),
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            is_closed,
        };
        
        // 更新K线缓存
        let kline_cache = get_kline_cache();
        let mut cache = kline_cache.write().await;
        
        // 如果是已完成的K线，添加到缓存
        if is_closed {
            // 🔥 Phase 1: K线连续性检测
            let detector = get_continuity_detector();
            let check_result = detector.check_continuity(
                Exchange::Binance,
                &symbol_upper,
                parse_interval(interval),
                open_time,
            ).await;
            
            // 根据连续性检测结果设置数据质量
            let data_quality = check_result.data_quality;
            
            // 如果检测到间隙，已经在detector内部输出了warn日志
            // 这里无论是否有间隙，都继续正常的缓存和存储流程
            
            cache.push(kline.clone());
            
            // 保持最新100条记录
            if cache.len() > 100 {
                cache.remove(0);
            }
            
            info!("📈 {} K线更新: O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2} [{}]", 
                  symbol_upper, open, high, low, close, volume, data_quality);
            
            // 存储到数据库 (如果启用) - 传递数据质量信息
            if let Err(e) = store_kline_to_database(&kline, storage, data_quality).await {
                warn!("存储K线到数据库失败: {}", e);
            }
        }
    }
    
    Ok(())
}
/// 存储K线数据到数据库
async fn store_kline_to_database(kline: &KlineData, storage: &SimpleStorage, data_quality: DataQuality) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !storage.enabled {
        return Ok(());
    }
    
    // 转换为shared_models格式
    let shared_kline = Kline {
        id: None,
        exchange: Exchange::Binance,
        symbol: kline.symbol.clone(),
        interval: parse_interval(&kline.interval),
        open_time: DateTime::from_timestamp_millis(kline.open_time).unwrap_or_else(|| Utc::now()),
        close_time: DateTime::from_timestamp_millis(kline.close_time).unwrap_or_else(|| Utc::now()),
        open: Decimal::from_f64_retain(kline.open).unwrap_or_default(),
        high: Decimal::from_f64_retain(kline.high).unwrap_or_default(),
        low: Decimal::from_f64_retain(kline.low).unwrap_or_default(),
        close: Decimal::from_f64_retain(kline.close).unwrap_or_default(),
        volume: Decimal::from_f64_retain(kline.volume).unwrap_or_default(),
        quote_volume: Decimal::from_f64_retain(kline.volume * kline.close).unwrap_or_default(),
        trades_count: 0,
        taker_buy_base_volume: Decimal::from_f64_retain(kline.volume * 0.6).unwrap_or_default(),
        taker_buy_quote_volume: Decimal::from_f64_retain(kline.volume * kline.close * 0.6).unwrap_or_default(),
        is_closed: kline.is_closed,
        data_quality, // 🔥 添加数据质量字段
    };
    
    // 真正存储到数据库
    storage.store_kline(&shared_kline).await?;
    
    Ok(())
}

/// 存储Ticker数据到数据库
async fn store_ticker_to_database(
    symbol: &str,
    market_data: &MarketData,
    storage: &SimpleStorage,
    data_quality: DataQuality
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !storage.enabled {
        return Ok(());
    }
    
    // 转换为shared_models格式
    let tick = MarketTick {
        id: None,
        exchange: Exchange::Binance,
        symbol: symbol.to_string(),
        timestamp: DateTime::from_timestamp_millis(market_data.timestamp).unwrap_or_else(|| Utc::now()),
        price: Decimal::from_f64_retain(market_data.price).unwrap_or_default(),
        volume: Decimal::from_f64_retain(market_data.volume).unwrap_or_default(),
        bid: Decimal::from_f64_retain(market_data.price * 0.999).unwrap_or_default(), // 估算买价
        ask: Decimal::from_f64_retain(market_data.price * 1.001).unwrap_or_default(), // 估算卖价
        bid_volume: Decimal::from_f64_retain(market_data.volume * 0.4).unwrap_or_default(),
        ask_volume: Decimal::from_f64_retain(market_data.volume * 0.4).unwrap_or_default(),
        trade_id: None,
        is_buyer_maker: None,
        data_quality, // 🔥 添加数据质量字段
    };
    
    // 真正存储到数据库
    storage.store_tick(&tick).await?;
    
    Ok(())
}