use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::MarketDataConfig;
use crate::storage::StorageManager;
use crate::processors::DataEvent;

/// 数据间隙检测器
pub struct DataGapDetector {
    config: MarketDataConfig,
    storage_manager: Arc<StorageManager>,
    last_timestamps: Arc<RwLock<HashMap<String, i64>>>,
    gap_threshold: Duration,
    stats: Arc<RwLock<GapDetectorStats>>,
    exchange_apis: HashMap<String, Box<dyn ExchangeRestAPI>>,
}

impl DataGapDetector {
    /// 创建新的间隙检测器
    pub async fn new(config: MarketDataConfig) -> Result<Self> {
        let gap_threshold = Duration::from_secs(config.data_processing.max_gap_seconds.unwrap_or(300)); // 默认5分钟
        
        // 初始化交易所REST API客户端
        let mut exchange_apis: HashMap<String, Box<dyn ExchangeRestAPI>> = HashMap::new();
        
        for (exchange_name, exchange_config) in &config.exchanges {
            if exchange_config.enabled {
                match exchange_name.as_str() {
                    "binance" => {
                        let api = BinanceRestAPI::new(exchange_config.clone())?;
                        exchange_apis.insert(exchange_name.clone(), Box::new(api));
                    }
                    "okx" => {
                        let api = OkxRestAPI::new(exchange_config.clone())?;
                        exchange_apis.insert(exchange_name.clone(), Box::new(api));
                    }
                    _ => {
                        warn!("Unsupported exchange for REST API: {}", exchange_name);
                    }
                }
            }
        }

        Ok(Self {
            config,
            storage_manager: Arc::new(StorageManager::new(config.clone()).await?),
            last_timestamps: Arc::new(RwLock::new(HashMap::new())),
            gap_threshold,
            stats: Arc::new(RwLock::new(GapDetectorStats::default())),
            exchange_apis,
        })
    }

    /// 检查数据间隙
    pub async fn check_gap(&self, symbol: &str, timestamp: i64) -> Result<()> {
        let mut timestamps = self.last_timestamps.write().await;
        
        if let Some(last_ts) = timestamps.get(symbol) {
            let gap_duration = Duration::from_millis((timestamp - last_ts) as u64);
            
            if gap_duration > self.gap_threshold {
                // 检测到数据间隙
                let mut stats = self.stats.write().await;
                stats.total_gaps_detected += 1;
                stats.last_gap_detected = Some(chrono::Utc::now().timestamp_millis());
                
                warn!(
                    "Data gap detected for {}: {}ms (threshold: {}ms)",
                    symbol,
                    gap_duration.as_millis(),
                    self.gap_threshold.as_millis()
                );

                // 尝试填补数据间隙
                match self.fill_data_gap(symbol, *last_ts, timestamp).await {
                    Ok(filled_count) => {
                        stats.total_gaps_filled += 1;
                        stats.total_records_filled += filled_count;
                        info!("Successfully filled data gap for {}: {} records", symbol, filled_count);
                    }
                    Err(e) => {
                        stats.failed_gap_fills += 1;
                        error!("Failed to fill data gap for {}: {}", symbol, e);
                    }
                }
            }
        }
        
        // 更新最后时间戳
        timestamps.insert(symbol.to_string(), timestamp);
        Ok(())
    }

    /// 填补数据间隙
    async fn fill_data_gap(&self, symbol: &str, start_ts: i64, end_ts: i64) -> Result<u64> {
        info!("Filling data gap for {} from {} to {}", symbol, start_ts, end_ts);
        
        // 解析交易所和交易对
        let (exchange, clean_symbol) = self.parse_symbol_info(symbol)?;
        
        // 获取对应的REST API客户端
        let api = self.exchange_apis.get(&exchange)
            .ok_or_else(|| anyhow::anyhow!("No REST API available for exchange: {}", exchange))?;

        let mut total_filled = 0u64;

        // 1. 填补K线数据
        for interval in &["1m", "5m", "15m", "1h"] {
            match api.get_klines(&clean_symbol, interval, start_ts, end_ts).await {
                Ok(klines) => {
                    for mut kline in klines {
                        // 标记为回填数据
                        kline.is_backfilled = true;
                        
                        // 直接存储到数据库，跳过Kafka避免重复处理
                        self.storage_manager.store_backfilled_kline(&kline).await?;
                        total_filled += 1;
                    }
                    
                    debug!("Filled {} {} klines for {}", total_filled, interval, symbol);
                }
                Err(e) => {
                    warn!("Failed to get {} klines for {}: {}", interval, symbol, e);
                }
            }
        }

        // 2. 填补交易数据（如果支持）
        if let Ok(trades) = api.get_trades(&clean_symbol, start_ts, end_ts).await {
            for mut trade in trades {
                trade.is_backfilled = true;
                self.storage_manager.store_backfilled_trade(&trade).await?;
                total_filled += 1;
            }
            
            debug!("Filled {} trades for {}", trades.len(), symbol);
        }

        Ok(total_filled)
    }

    /// 解析交易对信息
    fn parse_symbol_info(&self, symbol: &str) -> Result<(String, String)> {
        // 假设格式为 "exchange:symbol" 或直接是 "symbol"
        if let Some((exchange, symbol)) = symbol.split_once(':') {
            Ok((exchange.to_string(), symbol.to_string()))
        } else {
            // 如果没有指定交易所，使用默认的第一个启用的交易所
            let default_exchange = self.config.enabled_exchanges()
                .first()
                .map(|(name, _)| name.clone())
                .ok_or_else(|| anyhow::anyhow!("No enabled exchanges found"))?;
            
            Ok((default_exchange, symbol.to_string()))
        }
    }

    /// 批量检查多个交易对的间隙
    pub async fn batch_check_gaps(&self, events: &[DataEvent]) -> Result<()> {
        let mut symbol_timestamps: HashMap<String, i64> = HashMap::new();
        
        // 收集所有交易对的最新时间戳
        for event in events {
            if let Some(symbol) = event.symbol() {
                let timestamp = event.timestamp();
                symbol_timestamps.insert(symbol.to_string(), timestamp);
            }
        }
        
        // 并发检查所有交易对
        let mut tasks = Vec::new();
        
        for (symbol, timestamp) in symbol_timestamps {
            let detector = self.clone();
            let task = tokio::spawn(async move {
                if let Err(e) = detector.check_gap(&symbol, timestamp).await {
                    error!("Failed to check gap for {}: {}", symbol, e);
                }
            });
            tasks.push(task);
        }
        
        // 等待所有检查完成
        futures::future::join_all(tasks).await;
        
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> GapDetectorStats {
        self.stats.read().await.clone()
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        *self.stats.write().await = GapDetectorStats::default();
        info!("Gap detector statistics reset");
    }

    /// 获取所有交易对的最后时间戳
    pub async fn get_last_timestamps(&self) -> HashMap<String, i64> {
        self.last_timestamps.read().await.clone()
    }

    /// 手动触发间隙填补
    pub async fn manual_fill_gap(&self, symbol: &str, start_ts: i64, end_ts: i64) -> Result<u64> {
        info!("Manual gap fill requested for {} from {} to {}", symbol, start_ts, end_ts);
        
        let filled_count = self.fill_data_gap(symbol, start_ts, end_ts).await?;
        
        // 更新统计
        let mut stats = self.stats.write().await;
        stats.manual_fills += 1;
        stats.total_records_filled += filled_count;
        
        Ok(filled_count)
    }

    /// 关闭间隙检测器
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down gap detector");
        
        // 保存最终统计信息
        let stats = self.get_stats().await;
        info!("Gap detector final stats: {:?}", stats);
        
        Ok(())
    }
}

// 为了支持clone，需要实现Clone trait
impl Clone for DataGapDetector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            storage_manager: self.storage_manager.clone(),
            last_timestamps: self.last_timestamps.clone(),
            gap_threshold: self.gap_threshold,
            stats: self.stats.clone(),
            exchange_apis: HashMap::new(), // 简化处理，不克隆API客户端
        }
    }
}

/// 间隙检测器统计信息
#[derive(Debug, Clone, Default, Serialize)]
pub struct GapDetectorStats {
    pub total_gaps_detected: u64,
    pub total_gaps_filled: u64,
    pub failed_gap_fills: u64,
    pub total_records_filled: u64,
    pub manual_fills: u64,
    pub last_gap_detected: Option<i64>,
    pub last_consistency_check: Option<i64>,
}

impl GapDetectorStats {
    /// 计算填补成功率
    pub fn fill_success_rate(&self) -> f64 {
        if self.total_gaps_detected > 0 {
            self.total_gaps_filled as f64 / self.total_gaps_detected as f64 * 100.0
        } else {
            0.0
        }
    }

    /// 计算平均每次填补的记录数
    pub fn average_records_per_fill(&self) -> f64 {
        if self.total_gaps_filled > 0 {
            self.total_records_filled as f64 / self.total_gaps_filled as f64
        } else {
            0.0
        }
    }
}

/// 交易所REST API特征
#[async_trait::async_trait]
pub trait ExchangeRestAPI: Send + Sync {
    /// 获取K线数据
    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<shared_models::market::Kline>>;

    /// 获取交易数据
    async fn get_trades(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<shared_models::market::Trade>>;

    /// 获取最新价格
    async fn get_latest_price(&self, symbol: &str) -> Result<shared_models::market::MarketTick>;
}

/// 币安REST API实现
pub struct BinanceRestAPI {
    client: reqwest::Client,
    base_url: String,
}

impl BinanceRestAPI {
    pub fn new(config: crate::config::ExchangeConfig) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: "https://api.binance.com".to_string(),
        })
    }
}

#[async_trait::async_trait]
impl ExchangeRestAPI for BinanceRestAPI {
    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<shared_models::market::Kline>> {
        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&startTime={}&endTime={}&limit=1000",
            self.base_url, symbol, interval, start_time, end_time
        );

        let response = self.client.get(&url).send().await?;
        let data: Vec<serde_json::Value> = response.json().await?;

        let mut klines = Vec::new();
        for item in data {
            if let Some(array) = item.as_array() {
                if array.len() >= 11 {
                    let kline = shared_models::market::Kline {
                        exchange: "binance".to_string(),
                        symbol: symbol.to_string(),
                        interval: interval.to_string(),
                        open_time: array[0].as_i64().unwrap_or(0),
                        close_time: array[6].as_i64().unwrap_or(0),
                        open: rust_decimal::Decimal::from_str_exact(&array[1].as_str().unwrap_or("0"))?,
                        high: rust_decimal::Decimal::from_str_exact(&array[2].as_str().unwrap_or("0"))?,
                        low: rust_decimal::Decimal::from_str_exact(&array[3].as_str().unwrap_or("0"))?,
                        close: rust_decimal::Decimal::from_str_exact(&array[4].as_str().unwrap_or("0"))?,
                        volume: rust_decimal::Decimal::from_str_exact(&array[5].as_str().unwrap_or("0"))?,
                        quote_volume: rust_decimal::Decimal::from_str_exact(&array[7].as_str().unwrap_or("0"))?,
                        trade_count: array[8].as_u64().unwrap_or(0),
                        taker_buy_volume: rust_decimal::Decimal::from_str_exact(&array[9].as_str().unwrap_or("0"))?,
                        taker_buy_quote_volume: rust_decimal::Decimal::from_str_exact(&array[10].as_str().unwrap_or("0"))?,
                        is_closed: true,
                        is_backfilled: true,
                    };
                    klines.push(kline);
                }
            }
        }

        Ok(klines)
    }

    async fn get_trades(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<shared_models::market::Trade>> {
        // 币安的交易数据API实现
        // 这里简化处理，实际实现需要处理分页等
        Ok(Vec::new())
    }

    async fn get_latest_price(&self, symbol: &str) -> Result<shared_models::market::MarketTick> {
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.base_url, symbol);
        
        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let tick = shared_models::market::MarketTick {
            exchange: "binance".to_string(),
            symbol: symbol.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            price: rust_decimal::Decimal::from_str_exact(data["lastPrice"].as_str().unwrap_or("0"))?,
            volume: rust_decimal::Decimal::from_str_exact(data["volume"].as_str().unwrap_or("0"))?,
            bid: rust_decimal::Decimal::from_str_exact(data["bidPrice"].as_str().unwrap_or("0"))?,
            ask: rust_decimal::Decimal::from_str_exact(data["askPrice"].as_str().unwrap_or("0"))?,
        };

        Ok(tick)
    }
}

/// OKX REST API实现（占位符）
pub struct OkxRestAPI {
    client: reqwest::Client,
    base_url: String,
}

impl OkxRestAPI {
    pub fn new(config: crate::config::ExchangeConfig) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: "https://www.okx.com".to_string(),
        })
    }
}

#[async_trait::async_trait]
impl ExchangeRestAPI for OkxRestAPI {
    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<shared_models::market::Kline>> {
        // OKX API实现
        Ok(Vec::new())
    }

    async fn get_trades(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<shared_models::market::Trade>> {
        Ok(Vec::new())
    }

    async fn get_latest_price(&self, symbol: &str) -> Result<shared_models::market::MarketTick> {
        // OKX API实现
        Err(anyhow::anyhow!("OKX API not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_detector_stats() {
        let mut stats = GapDetectorStats::default();
        
        stats.total_gaps_detected = 10;
        stats.total_gaps_filled = 8;
        stats.total_records_filled = 1000;
        
        assert_eq!(stats.fill_success_rate(), 80.0);
        assert_eq!(stats.average_records_per_fill(), 125.0);
    }

    #[tokio::test]
    async fn test_parse_symbol_info() {
        // 这里需要创建一个测试用的配置
        // 实际测试中需要mock配置
    }
}