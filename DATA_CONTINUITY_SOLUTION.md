# 🛡️ 数据连续性和故障恢复解决方案

## 🎯 核心问题分析

### 开发阶段常见问题：
1. **服务重启** → 数据丢失
2. **网络中断** → 数据断层  
3. **系统崩溃** → 状态丢失
4. **数据不一致** → Redis和ClickHouse数据差异
5. **冷启动** → 缓存为空，查询慢

## 🔄 完整解决方案

### 1. **Kafka Offset管理 - 解决重启数据丢失**

```rust
// 消费者组配置 - 确保消息不丢失
pub struct KafkaConsumerConfig {
    pub group_id: String,
    pub auto_offset_reset: "earliest",  // 从最早消息开始
    pub enable_auto_commit: false,      // 手动提交offset
    pub session_timeout_ms: 30000,
    pub max_poll_records: 1000,
}

// 手动提交offset确保数据处理完成
impl DataProcessor {
    pub async fn process_with_offset_management(&self) -> Result<()> {
        let mut consumer = self.create_consumer().await?;
        
        loop {
            let messages = consumer.poll(Duration::from_millis(1000)).await?;
            
            for message in messages {
                // 1. 处理消息
                match self.process_message(&message).await {
                    Ok(_) => {
                        // 2. 只有处理成功才提交offset
                        consumer.commit_message(&message).await?;
                        info!("Message processed and committed: offset {}", message.offset());
                    }
                    Err(e) => {
                        // 3. 处理失败，记录错误但不提交offset
                        error!("Message processing failed: {}, will retry", e);
                        // 消息会在下次重启时重新处理
                    }
                }
            }
        }
    }
}
```

### 2. **数据断层检测和补齐机制**

```rust
// 数据连续性检查器
pub struct DataContinuityChecker {
    last_timestamps: HashMap<String, i64>,  // 每个交易对的最后时间戳
    gap_threshold: Duration,                // 数据断层阈值
    recovery_api: ExchangeRestAPI,          // REST API补齐数据
}

impl DataContinuityChecker {
    // 检测数据断层
    pub async fn check_data_gap(&mut self, symbol: &str, timestamp: i64) -> Result<()> {
        if let Some(last_ts) = self.last_timestamps.get(symbol) {
            let gap = Duration::from_millis((timestamp - last_ts) as u64);
            
            if gap > self.gap_threshold {
                warn!("Data gap detected for {}: {}ms", symbol, gap.as_millis());
                
                // 自动补齐数据
                self.fill_data_gap(symbol, *last_ts, timestamp).await?;
            }
        }
        
        self.last_timestamps.insert(symbol.to_string(), timestamp);
        Ok(())
    }
    
    // 补齐数据断层
    async fn fill_data_gap(&self, symbol: &str, start_ts: i64, end_ts: i64) -> Result<()> {
        info!("Filling data gap for {} from {} to {}", symbol, start_ts, end_ts);
        
        // 1. 从交易所REST API获取历史数据
        let historical_data = self.recovery_api.get_klines(
            symbol,
            "1m",
            start_ts,
            end_ts
        ).await?;
        
        // 2. 标记为补齐数据并存储
        for mut kline in historical_data {
            kline.is_backfilled = true;  // 标记为补齐数据
            
            // 3. 直接写入存储，跳过Kafka（避免重复处理）
            self.storage_manager.store_backfilled_data(&kline).await?;
        }
        
        info!("Data gap filled: {} records for {}", historical_data.len(), symbol);
        Ok(())
    }
}
```

### 3. **服务状态持久化 - 解决重启状态丢失**

```rust
// 服务状态管理器
pub struct ServiceStateManager {
    state_file: PathBuf,
    redis: Arc<RedisStorage>,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceState {
    pub last_processed_timestamps: HashMap<String, i64>,
    pub active_subscriptions: Vec<String>,
    pub consumer_offsets: HashMap<String, i64>,
    pub startup_time: i64,
    pub shutdown_time: Option<i64>,
}

impl ServiceStateManager {
    // 保存服务状态
    pub async fn save_state(&self, state: &ServiceState) -> Result<()> {
        // 1. 保存到本地文件
        let state_json = serde_json::to_string_pretty(state)?;
        tokio::fs::write(&self.state_file, state_json).await?;
        
        // 2. 备份到Redis
        self.redis.set("service_state:market_data", state, 86400).await?;
        
        debug!("Service state saved");
        Ok(())
    }
    
    // 恢复服务状态
    pub async fn load_state(&self) -> Result<Option<ServiceState>> {
        // 1. 优先从本地文件恢复
        if let Ok(state_json) = tokio::fs::read_to_string(&self.state_file).await {
            if let Ok(state) = serde_json::from_str::<ServiceState>(&state_json) {
                info!("Service state loaded from file");
                return Ok(Some(state));
            }
        }
        
        // 2. 从Redis恢复
        if let Ok(Some(state)) = self.redis.get::<ServiceState>("service_state:market_data").await {
            info!("Service state loaded from Redis");
            return Ok(Some(state));
        }
        
        warn!("No previous service state found, starting fresh");
        Ok(None)
    }
    
    // 定期保存状态
    pub fn start_periodic_save(&self, state: Arc<RwLock<ServiceState>>) {
        let manager = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let current_state = state.read().await.clone();
                if let Err(e) = manager.save_state(&current_state).await {
                    error!("Failed to save service state: {}", e);
                }
            }
        });
    }
}
```

### 4. **优雅关机和启动机制**

```rust
// 优雅关机处理器
pub struct GracefulShutdownHandler {
    shutdown_signal: Arc<tokio::sync::Notify>,
    services: Vec<Box<dyn ShutdownService>>,
}

#[async_trait]
pub trait ShutdownService: Send + Sync {
    async fn shutdown(&mut self) -> Result<()>;
}

impl GracefulShutdownHandler {
    pub async fn wait_for_shutdown(&self) {
        // 监听关机信号
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, initiating graceful shutdown");
            }
            _ = self.shutdown_signal.notified() => {
                info!("Received shutdown signal");
            }
        }
        
        // 执行优雅关机
        self.perform_shutdown().await;
    }
    
    async fn perform_shutdown(&self) {
        info!("Starting graceful shutdown...");
        
        // 1. 停止接收新数据
        info!("Stopping data ingestion...");
        
        // 2. 处理完所有缓冲区数据
        info!("Flushing all buffers...");
        
        // 3. 保存服务状态
        info!("Saving service state...");
        
        // 4. 关闭所有服务
        for (i, service) in self.services.iter().enumerate() {
            info!("Shutting down service {}...", i);
            if let Err(e) = service.shutdown().await {
                error!("Failed to shutdown service {}: {}", i, e);
            }
        }
        
        info!("Graceful shutdown completed");
    }
}

// 市场数据服务的关机实现
#[async_trait]
impl ShutdownService for MarketDataService {
    async fn shutdown(&mut self) -> Result<()> {
        // 1. 停止WebSocket连接
        self.exchange_manager.disconnect_all().await?;
        
        // 2. 刷新所有缓冲区
        self.data_processor.flush_all_buffers().await?;
        self.storage_manager.flush_all().await?;
        
        // 3. 提交Kafka offset
        self.kafka_consumer.commit_sync().await?;
        
        // 4. 保存最后状态
        let state = ServiceState {
            last_processed_timestamps: self.get_last_timestamps(),
            shutdown_time: Some(chrono::Utc::now().timestamp_millis()),
            ..Default::default()
        };
        self.state_manager.save_state(&state).await?;
        
        // 5. 关闭存储连接
        self.storage_manager.shutdown().await?;
        
        Ok(())
    }
}
```

### 5. **数据一致性检查和修复**

```rust
// 数据一致性检查器
pub struct DataConsistencyChecker {
    redis: Arc<RedisStorage>,
    clickhouse: Arc<ClickHouseStorage>,
}

impl DataConsistencyChecker {
    // 检查Redis和ClickHouse数据一致性
    pub async fn check_consistency(&self, symbol: &str) -> Result<ConsistencyReport> {
        let mut report = ConsistencyReport::new(symbol);
        
        // 1. 获取Redis中的最新数据
        let redis_tick = self.redis.get_latest_tick(symbol).await?;
        
        // 2. 获取ClickHouse中的最新数据
        let ch_tick = self.clickhouse.get_latest_tick(symbol).await?;
        
        // 3. 比较时间戳
        match (redis_tick, ch_tick) {
            (Some(redis), Some(ch)) => {
                let time_diff = (redis.timestamp - ch.timestamp).abs();
                
                if time_diff > 60000 { // 超过1分钟差异
                    report.add_issue(ConsistencyIssue::TimestampMismatch {
                        redis_ts: redis.timestamp,
                        clickhouse_ts: ch.timestamp,
                        diff_ms: time_diff,
                    });
                }
                
                // 4. 比较价格
                if (redis.price - ch.price).abs() > Decimal::new(1, 4) { // 0.0001差异
                    report.add_issue(ConsistencyIssue::PriceMismatch {
                        redis_price: redis.price,
                        clickhouse_price: ch.price,
                    });
                }
            }
            (Some(_), None) => {
                report.add_issue(ConsistencyIssue::MissingInClickHouse);
            }
            (None, Some(_)) => {
                report.add_issue(ConsistencyIssue::MissingInRedis);
            }
            (None, None) => {
                report.add_issue(ConsistencyIssue::NoDataFound);
            }
        }
        
        Ok(report)
    }
    
    // 修复数据不一致
    pub async fn repair_inconsistency(&self, symbol: &str) -> Result<()> {
        let report = self.check_consistency(symbol).await?;
        
        for issue in report.issues {
            match issue {
                ConsistencyIssue::MissingInRedis => {
                    // 从ClickHouse同步到Redis
                    if let Some(tick) = self.clickhouse.get_latest_tick(symbol).await? {
                        self.redis.store_tick(&tick).await?;
                        info!("Synced missing data from ClickHouse to Redis for {}", symbol);
                    }
                }
                ConsistencyIssue::MissingInClickHouse => {
                    // 从Redis同步到ClickHouse
                    if let Some(tick) = self.redis.get_latest_tick(symbol).await? {
                        self.clickhouse.store_tick(&tick).await?;
                        info!("Synced missing data from Redis to ClickHouse for {}", symbol);
                    }
                }
                ConsistencyIssue::TimestampMismatch { .. } => {
                    // 以ClickHouse为准，更新Redis
                    if let Some(tick) = self.clickhouse.get_latest_tick(symbol).await? {
                        self.redis.store_tick(&tick).await?;
                        info!("Fixed timestamp mismatch for {}", symbol);
                    }
                }
                _ => {
                    warn!("Cannot auto-repair issue: {:?}", issue);
                }
            }
        }
        
        Ok(())
    }
}
```

### 6. **启动时数据预热机制**

```rust
// 数据预热器
pub struct DataPreloader {
    redis: Arc<RedisStorage>,
    clickhouse: Arc<ClickHouseStorage>,
}

impl DataPreloader {
    // 启动时预热缓存
    pub async fn preload_cache(&self, symbols: &[String]) -> Result<()> {
        info!("Starting cache preload for {} symbols", symbols.len());
        
        let mut tasks = Vec::new();
        
        for symbol in symbols {
            let symbol = symbol.clone();
            let redis = self.redis.clone();
            let clickhouse = self.clickhouse.clone();
            
            let task = tokio::spawn(async move {
                // 1. 预热最新Tick数据
                if let Ok(Some(tick)) = clickhouse.get_latest_tick(&symbol).await {
                    let _ = redis.store_tick(&tick).await;
                }
                
                // 2. 预热最新K线数据
                for interval in &["1m", "5m", "15m", "1h", "4h", "1d"] {
                    if let Ok(Some(kline)) = clickhouse.get_latest_kline(&symbol, interval).await {
                        let _ = redis.store_kline(&kline).await;
                    }
                }
                
                info!("Cache preloaded for {}", symbol);
            });
            
            tasks.push(task);
        }
        
        // 等待所有预热任务完成
        futures::future::join_all(tasks).await;
        
        info!("Cache preload completed");
        Ok(())
    }
}
```

## 🚀 完整启动流程

```rust
// 主启动函数
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化日志
    LoggingInitializer::init_dev()?;
    
    // 2. 加载配置
    let config = MarketDataConfig::load()?;
    
    // 3. 恢复服务状态
    let state_manager = ServiceStateManager::new("./data/service_state.json");
    let previous_state = state_manager.load_state().await?;
    
    // 4. 初始化存储
    let storage_manager = Arc::new(StorageManager::new(config.clone()).await?);
    
    // 5. 数据一致性检查
    let consistency_checker = DataConsistencyChecker::new(
        storage_manager.get_redis(),
        storage_manager.get_clickhouse()
    );
    
    for symbol in &config.all_symbols() {
        if let Err(e) = consistency_checker.repair_inconsistency(symbol).await {
            warn!("Failed to repair consistency for {}: {}", symbol, e);
        }
    }
    
    // 6. 预热缓存
    let preloader = DataPreloader::new(
        storage_manager.get_redis(),
        storage_manager.get_clickhouse()
    );
    preloader.preload_cache(&config.all_symbols()).await?;
    
    // 7. 初始化数据处理器（从上次offset继续）
    let mut data_processor = DataProcessor::new(config.clone(), storage_manager.clone()).await?;
    if let Some(state) = previous_state {
        data_processor.restore_from_state(&state).await?;
    }
    
    // 8. 启动数据连续性检查
    let continuity_checker = DataContinuityChecker::new(config.clone());
    data_processor.set_continuity_checker(continuity_checker);
    
    // 9. 启动服务
    let exchange_manager = Arc::new(ExchangeManager::new(config.clone()).await?);
    exchange_manager.start_all_connections().await?;
    
    // 10. 设置优雅关机
    let shutdown_handler = GracefulShutdownHandler::new();
    shutdown_handler.add_service(Box::new(data_processor));
    shutdown_handler.add_service(Box::new(exchange_manager));
    
    // 11. 启动定期状态保存
    let current_state = Arc::new(RwLock::new(ServiceState::new()));
    state_manager.start_periodic_save(current_state.clone());
    
    info!("🚀 Market Data Service started successfully");
    
    // 12. 等待关机信号
    shutdown_handler.wait_for_shutdown().await;
    
    Ok(())
}
```

## 🎯 开发阶段使用指南

### 日常开发流程：
```bash
# 1. 启动服务（自动恢复状态）
cargo run

# 2. 开发调试（服务继续运行）
# 修改代码...

# 3. 重启服务（Ctrl+C优雅关机，自动保存状态）
# 重新启动会从上次状态继续

# 4. 检查数据一致性
curl http://localhost:8081/api/v1/admin/consistency-check

# 5. 手动修复数据
curl -X POST http://localhost:8081/api/v1/admin/repair-data
```

这样的设计确保了：
- ✅ **零数据丢失**：Kafka offset管理 + 状态持久化
- ✅ **自动恢复**：断层检测 + REST API补齐
- ✅ **数据一致性**：定期检查 + 自动修复
- ✅ **快速启动**：缓存预热 + 状态恢复
- ✅ **开发友好**：优雅关机 + 状态保存