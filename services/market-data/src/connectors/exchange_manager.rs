use anyhow::Result;
use shared_utils::AppMetrics;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::config::MarketDataConfig;
use crate::processors::DataProcessor;

use super::{
    BinanceConnector, ExchangeConnector, MarketDataEvent, ConnectionStats,
    ConnectorError, BinanceConfig,
};

/// 交易所管理器
pub struct ExchangeManager {
    config: MarketDataConfig,
    connectors: Arc<RwLock<HashMap<String, Box<dyn ExchangeConnector + Send + Sync>>>>,
    event_sender: mpsc::UnboundedSender<MarketDataEvent>,
    data_processor: Arc<DataProcessor>,
    metrics: Arc<AppMetrics>,
    stats: Arc<RwLock<ExchangeManagerStats>>,
}

/// 交易所管理器统计信息
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ExchangeManagerStats {
    pub total_connectors: usize,
    pub connected_connectors: usize,
    pub total_events_processed: u64,
    pub events_per_second: f64,
    pub last_event_time: Option<chrono::DateTime<chrono::Utc>>,
    pub connector_stats: HashMap<String, ConnectionStats>,
}

impl ExchangeManager {
    /// 创建新的交易所管理器
    pub async fn new(
        config: MarketDataConfig,
        data_processor: Arc<DataProcessor>,
        metrics: Arc<AppMetrics>,
    ) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let manager = Self {
            config,
            connectors: Arc::new(RwLock::new(HashMap::new())),
            event_sender: event_sender.clone(),
            data_processor: data_processor.clone(),
            metrics: metrics.clone(),
            stats: Arc::new(RwLock::new(ExchangeManagerStats::default())),
        };

        // 启动事件处理任务
        manager.start_event_processor(event_receiver).await;

        Ok(manager)
    }

    /// 启动所有配置的交易所连接
    pub async fn start_all_connections(&self) -> Result<()> {
        info!("Starting all exchange connections");

        for (exchange_name, exchange_config) in self.config.enabled_exchanges() {
            if let Err(e) = self.start_exchange_connection(exchange_name, exchange_config).await {
                error!("Failed to start connection for {}: {}", exchange_name, e);
                // 继续启动其他交易所，不因为一个失败而停止
            }
        }

        info!("All exchange connections started");
        Ok(())
    }

    /// 启动单个交易所连接
    async fn start_exchange_connection(
        &self,
        exchange_name: &str,
        exchange_config: &crate::config::ExchangeConfig,
    ) -> Result<()> {
        info!("Starting connection for exchange: {}", exchange_name);

        match exchange_name {
            "binance" => {
                self.start_binance_connection(exchange_config).await?;
            }
            "okx" => {
                // TODO: 实现OKX连接器
                warn!("OKX connector not implemented yet");
            }
            "huobi" => {
                // TODO: 实现火币连接器
                warn!("Huobi connector not implemented yet");
            }
            _ => {
                warn!("Unknown exchange: {}", exchange_name);
                return Err(ConnectorError::ConfigurationError(
                    format!("Unsupported exchange: {}", exchange_name)
                ).into());
            }
        }

        Ok(())
    }

    /// 启动币安连接
    async fn start_binance_connection(
        &self,
        exchange_config: &crate::config::ExchangeConfig,
    ) -> Result<()> {
        let binance_config = BinanceConfig {
            websocket_url: exchange_config.websocket_url.clone(),
            testnet: false, // 从配置中读取
            auto_reconnect: true,
            max_reconnect_attempts: exchange_config.connection.max_reconnect_attempts,
            reconnect_interval: std::time::Duration::from_secs(
                exchange_config.connection.reconnect_interval
            ),
        };

        let mut connector = BinanceConnector::new(
            exchange_config.symbols.clone(),
            self.event_sender.clone(),
            binance_config,
        );

        // 连接到币安
        connector.connect().await?;

        // 订阅数据
        if exchange_config.data_types.ticker {
            connector.subscribe(&exchange_config.symbols, &["ticker".to_string()]).await?;
        }

        if exchange_config.data_types.kline {
            for interval in &exchange_config.data_types.kline_intervals {
                let data_type = format!("kline_{}", interval);
                connector.subscribe(&exchange_config.symbols, &[data_type]).await?;
            }
        }

        if exchange_config.data_types.depth {
            connector.subscribe(&exchange_config.symbols, &["depth".to_string()]).await?;
        }

        if exchange_config.data_types.trade {
            connector.subscribe(&exchange_config.symbols, &["trade".to_string()]).await?;
        }

        // 将连接器添加到管理器
        {
            let mut connectors = self.connectors.write().await;
            connectors.insert("binance".to_string(), Box::new(connector));
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_connectors += 1;
            stats.connected_connectors += 1;
        }

        info!("Binance connection started successfully");
        Ok(())
    }

    /// 启动事件处理器
    async fn start_event_processor(
        &self,
        mut event_receiver: mpsc::UnboundedReceiver<MarketDataEvent>,
    ) {
        let data_processor = self.data_processor.clone();
        let metrics = self.metrics.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            info!("Exchange manager event processor started");

            while let Some(event) = event_receiver.recv().await {
                let start_time = std::time::Instant::now();

                // 更新统计信息
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.total_events_processed += 1;
                    stats_guard.last_event_time = Some(chrono::Utc::now());
                }

                // 处理事件
                match Self::process_market_event(&event, &data_processor, &metrics).await {
                    Ok(_) => {
                        let processing_time = start_time.elapsed();
                        debug!(
                            "Event processed successfully: {} in {:?}",
                            event.event_type(),
                            processing_time
                        );

                        // 记录处理时间指标
                        let _ = metrics.record_histogram(
                            "market_data_event_processing_duration_seconds",
                            processing_time.as_secs_f64(),
                        );
                    }
                    Err(e) => {
                        error!("Failed to process event: {} - {:?}", e, event);
                        let _ = metrics.inc_counter("market_data_event_processing_errors_total");
                    }
                }
            }

            warn!("Exchange manager event processor stopped");
        });
    }

    /// 处理市场数据事件
    async fn process_market_event(
        event: &MarketDataEvent,
        data_processor: &DataProcessor,
        metrics: &AppMetrics,
    ) -> Result<()> {
        match event {
            MarketDataEvent::Tick(tick) => {
                debug!("Processing tick: {} {}", tick.exchange, tick.symbol);
                data_processor.process_tick(tick).await?;
                let _ = metrics.inc_counter_vec("market_data_ticks_total", &[&tick.exchange, &tick.symbol]);
            }
            MarketDataEvent::Kline(kline) => {
                debug!("Processing kline: {} {} {}", kline.exchange, kline.symbol, kline.interval);
                data_processor.process_kline(kline).await?;
                let _ = metrics.inc_counter_vec("market_data_klines_total", &[&kline.exchange, &kline.symbol]);
            }
            MarketDataEvent::OrderBook(orderbook) => {
                debug!("Processing orderbook: {} {}", orderbook.exchange, orderbook.symbol);
                data_processor.process_orderbook(orderbook).await?;
                let _ = metrics.inc_counter_vec("market_data_orderbooks_total", &[&orderbook.exchange, &orderbook.symbol]);
            }
            MarketDataEvent::Trade(trade) => {
                debug!("Processing trade: {} {}", trade.exchange, trade.symbol);
                data_processor.process_trade(trade).await?;
                let _ = metrics.inc_counter_vec("market_data_trades_total", &[&trade.exchange, &trade.symbol]);
            }
            MarketDataEvent::Heartbeat { exchange, .. } => {
                debug!("Processing heartbeat from: {}", exchange);
                let _ = metrics.inc_counter_vec("market_data_heartbeats_total", &[exchange]);
            }
            MarketDataEvent::Error { exchange, error, .. } => {
                warn!("Processing error from {}: {}", exchange, error);
                let _ = metrics.inc_counter_vec("market_data_errors_total", &[exchange]);
            }
            MarketDataEvent::ConnectionStatus { exchange, connected, .. } => {
                info!("Connection status for {}: {}", exchange, connected);
                let status = if *connected { "connected" } else { "disconnected" };
                let _ = metrics.inc_counter_vec("market_data_connection_status_total", &[exchange, status]);
            }
        }

        Ok(())
    }

    /// 停止所有连接
    pub async fn stop_all_connections(&self) -> Result<()> {
        info!("Stopping all exchange connections");

        let mut connectors = self.connectors.write().await;
        
        for (exchange_name, connector) in connectors.iter_mut() {
            info!("Stopping connection for: {}", exchange_name);
            
            if let Err(e) = connector.disconnect().await {
                error!("Failed to disconnect {}: {}", exchange_name, e);
            }
        }

        connectors.clear();

        // 重置统计信息
        {
            let mut stats = self.stats.write().await;
            stats.connected_connectors = 0;
        }

        info!("All exchange connections stopped");
        Ok(())
    }

    /// 重启指定交易所连接
    pub async fn restart_exchange_connection(&self, exchange_name: &str) -> Result<()> {
        info!("Restarting connection for: {}", exchange_name);

        // 先停止现有连接
        {
            let mut connectors = self.connectors.write().await;
            if let Some(mut connector) = connectors.remove(exchange_name) {
                if let Err(e) = connector.disconnect().await {
                    warn!("Error disconnecting {}: {}", exchange_name, e);
                }
            }
        }

        // 重新启动连接
        if let Some(exchange_config) = self.config.exchanges.get(exchange_name) {
            if exchange_config.enabled {
                self.start_exchange_connection(exchange_name, exchange_config).await?;
                info!("Connection restarted successfully for: {}", exchange_name);
            } else {
                warn!("Exchange {} is disabled in configuration", exchange_name);
            }
        } else {
            return Err(ConnectorError::ConfigurationError(
                format!("Exchange {} not found in configuration", exchange_name)
            ).into());
        }

        Ok(())
    }

    /// 获取所有连接器的统计信息
    pub async fn get_all_stats(&self) -> ExchangeManagerStats {
        let mut stats = self.stats.read().await.clone();
        
        // 更新连接器统计信息
        let connectors = self.connectors.read().await;
        stats.connector_stats.clear();
        
        for (name, connector) in connectors.iter() {
            stats.connector_stats.insert(name.clone(), connector.get_stats());
        }

        // 计算每秒事件数
        if let Some(last_event_time) = stats.last_event_time {
            let duration = chrono::Utc::now() - last_event_time;
            let seconds = duration.num_seconds() as f64;
            if seconds > 0.0 {
                stats.events_per_second = stats.total_events_processed as f64 / seconds;
            }
        }

        stats
    }

    /// 获取指定交易所的统计信息
    pub async fn get_exchange_stats(&self, exchange_name: &str) -> Option<ConnectionStats> {
        let connectors = self.connectors.read().await;
        connectors.get(exchange_name).map(|connector| connector.get_stats())
    }

    /// 检查所有连接状态
    pub async fn check_all_connections(&self) -> HashMap<String, bool> {
        let mut connection_status = HashMap::new();
        let connectors = self.connectors.read().await;
        
        for (name, connector) in connectors.iter() {
            connection_status.insert(name.clone(), connector.is_connected());
        }
        
        connection_status
    }

    /// 订阅新的数据流
    pub async fn subscribe_data(
        &self,
        exchange_name: &str,
        symbols: &[String],
        data_types: &[String],
    ) -> Result<()> {
        info!("Subscribing to data: exchange={}, symbols={:?}, types={:?}", 
              exchange_name, symbols, data_types);

        let mut connectors = self.connectors.write().await;
        
        if let Some(connector) = connectors.get_mut(exchange_name) {
            connector.subscribe(symbols, data_types).await?;
            info!("Subscription successful for: {}", exchange_name);
        } else {
            return Err(ConnectorError::ConnectionFailed(
                format!("Exchange {} not connected", exchange_name)
            ).into());
        }

        Ok(())
    }

    /// 取消订阅数据流
    pub async fn unsubscribe_data(
        &self,
        exchange_name: &str,
        symbols: &[String],
        data_types: &[String],
    ) -> Result<()> {
        info!("Unsubscribing from data: exchange={}, symbols={:?}, types={:?}", 
              exchange_name, symbols, data_types);

        let mut connectors = self.connectors.write().await;
        
        if let Some(connector) = connectors.get_mut(exchange_name) {
            connector.unsubscribe(symbols, data_types).await?;
            info!("Unsubscription successful for: {}", exchange_name);
        } else {
            return Err(ConnectorError::ConnectionFailed(
                format!("Exchange {} not connected", exchange_name)
            ).into());
        }

        Ok(())
    }

    /// 获取支持的交易所列表
    pub fn get_supported_exchanges(&self) -> Vec<String> {
        self.config.exchanges.keys().cloned().collect()
    }

    /// 获取交易所支持的交易对
    pub async fn get_exchange_symbols(&self, exchange_name: &str) -> Option<Vec<String>> {
        let connectors = self.connectors.read().await;
        connectors.get(exchange_name)
            .map(|connector| connector.supported_symbols().to_vec())
    }

    /// 健康检查
    pub async fn health_check(&self) -> ExchangeManagerHealth {
        let stats = self.get_all_stats().await;
        let connection_status = self.check_all_connections().await;
        
        let healthy_connections = connection_status.values().filter(|&&connected| connected).count();
        let total_connections = connection_status.len();
        
        let is_healthy = if total_connections > 0 {
            (healthy_connections as f64 / total_connections as f64) >= 0.5 // 至少50%连接正常
        } else {
            false
        };

        ExchangeManagerHealth {
            is_healthy,
            total_connections,
            healthy_connections,
            total_events_processed: stats.total_events_processed,
            events_per_second: stats.events_per_second,
            connection_status,
        }
    }
}

/// 交易所管理器健康状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExchangeManagerHealth {
    pub is_healthy: bool,
    pub total_connections: usize,
    pub healthy_connections: usize,
    pub total_events_processed: u64,
    pub events_per_second: f64,
    pub connection_status: HashMap<String, bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MarketDataConfig, DataProcessingConfig, WebSocketConfig, MonitoringConfig};
    use crate::storage::StorageManager;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_exchange_manager_creation() {
        let config = MarketDataConfig {
            server: crate::config::ServerConfig::default(),
            exchanges: HashMap::new(),
            storage: crate::config::StorageConfig::default(),
            data_processing: DataProcessingConfig::default(),
            websocket: WebSocketConfig::default(),
            monitoring: MonitoringConfig::default(),
        };

        let metrics = Arc::new(AppMetrics::new().unwrap());
        let storage_manager = Arc::new(
            StorageManager::new(config.clone()).await.unwrap()
        );
        let data_processor = Arc::new(
            DataProcessor::new(config.clone(), storage_manager, metrics.clone()).await.unwrap()
        );

        let manager = ExchangeManager::new(config, data_processor, metrics).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = MarketDataConfig {
            server: crate::config::ServerConfig::default(),
            exchanges: HashMap::new(),
            storage: crate::config::StorageConfig::default(),
            data_processing: DataProcessingConfig::default(),
            websocket: WebSocketConfig::default(),
            monitoring: MonitoringConfig::default(),
        };

        let metrics = Arc::new(AppMetrics::new().unwrap());
        let storage_manager = Arc::new(
            StorageManager::new(config.clone()).await.unwrap()
        );
        let data_processor = Arc::new(
            DataProcessor::new(config.clone(), storage_manager, metrics.clone()).await.unwrap()
        );

        let manager = ExchangeManager::new(config, data_processor, metrics).await.unwrap();
        let health = manager.health_check().await;
        
        // 没有连接时应该是不健康的
        assert!(!health.is_healthy);
        assert_eq!(health.total_connections, 0);
    }
}