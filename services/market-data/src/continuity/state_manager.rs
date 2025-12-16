use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, info, warn};

use crate::storage::RedisStorage;

/// 服务状态管理器
#[derive(Clone)]
pub struct ServiceStateManager {
    state_file: PathBuf,
    redis: Option<RedisStorage>,
}

impl ServiceStateManager {
    /// 创建新的状态管理器
    pub fn new(state_file: PathBuf) -> Result<Self> {
        // 确保状态文件目录存在
        if let Some(parent) = state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            state_file,
            redis: None,
        })
    }

    /// 设置Redis备份
    pub fn with_redis(mut self, redis: RedisStorage) -> Self {
        self.redis = Some(redis);
        self
    }

    /// 保存服务状态
    pub async fn save_state(&self, state: &ServiceState) -> Result<()> {
        // 1. 保存到本地文件
        let state_json = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_file, state_json).await?;

        // 2. 备份到Redis（如果可用）
        if let Some(redis) = &self.redis {
            if let Err(e) = redis.set("service_state:market_data", state, 86400).await {
                warn!("Failed to backup state to Redis: {}", e);
            }
        }

        debug!("Service state saved to {:?}", self.state_file);
        Ok(())
    }

    /// 加载服务状态
    pub async fn load_state(&self) -> Result<Option<ServiceState>> {
        // 1. 优先从本地文件加载
        if let Ok(state_json) = fs::read_to_string(&self.state_file).await {
            if let Ok(state) = serde_json::from_str::<ServiceState>(&state_json) {
                info!("Service state loaded from file: {:?}", self.state_file);
                return Ok(Some(state));
            }
        }

        // 2. 从Redis恢复
        if let Some(redis) = &self.redis {
            if let Ok(Some(state)) = redis.get::<ServiceState>("service_state:market_data").await {
                info!("Service state loaded from Redis backup");
                
                // 同步到本地文件
                if let Err(e) = self.save_state(&state).await {
                    warn!("Failed to sync state from Redis to file: {}", e);
                }
                
                return Ok(Some(state));
            }
        }

        info!("No previous service state found");
        Ok(None)
    }
}
/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub startup_time: i64,
    pub shutdown_time: Option<i64>,
    pub last_update_time: i64,
    pub last_processed_timestamps: HashMap<String, i64>,
    pub active_subscriptions: Vec<String>,
    pub consumer_offsets: HashMap<String, i64>,
    pub service_version: String,
    pub configuration_hash: String,
}

impl ServiceState {
    /// 创建新的服务状态
    pub fn new() -> Self {
        Self {
            startup_time: chrono::Utc::now().timestamp_millis(),
            shutdown_time: None,
            last_update_time: chrono::Utc::now().timestamp_millis(),
            last_processed_timestamps: HashMap::new(),
            active_subscriptions: Vec::new(),
            consumer_offsets: HashMap::new(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            configuration_hash: "".to_string(),
        }
    }

    /// 计算运行时间（秒）
    pub fn uptime_seconds(&self) -> u64 {
        let end_time = self.shutdown_time.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        ((end_time - self.startup_time) / 1000) as u64
    }

    /// 更新最后处理时间戳
    pub fn update_timestamp(&mut self, symbol: &str, timestamp: i64) {
        self.last_processed_timestamps.insert(symbol.to_string(), timestamp);
        self.last_update_time = chrono::Utc::now().timestamp_millis();
    }

    /// 添加订阅
    pub fn add_subscription(&mut self, subscription: String) {
        if !self.active_subscriptions.contains(&subscription) {
            self.active_subscriptions.push(subscription);
        }
    }

    /// 移除订阅
    pub fn remove_subscription(&mut self, subscription: &str) {
        self.active_subscriptions.retain(|s| s != subscription);
    }

    /// 更新消费者偏移量
    pub fn update_consumer_offset(&mut self, topic: &str, offset: i64) {
        self.consumer_offsets.insert(topic.to_string(), offset);
    }

    /// 获取指定交易对的最后时间戳
    pub fn get_last_timestamp(&self, symbol: &str) -> Option<i64> {
        self.last_processed_timestamps.get(symbol).copied()
    }

    /// 检查是否有数据间隙
    pub fn has_potential_gaps(&self, max_gap_minutes: i64) -> Vec<String> {
        let now = chrono::Utc::now().timestamp_millis();
        let max_gap_ms = max_gap_minutes * 60 * 1000;
        
        self.last_processed_timestamps
            .iter()
            .filter_map(|(symbol, &timestamp)| {
                if now - timestamp > max_gap_ms {
                    Some(symbol.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 验证状态完整性
    pub fn validate(&self) -> Result<()> {
        if self.startup_time <= 0 {
            return Err(anyhow::anyhow!("Invalid startup time"));
        }

        if let Some(shutdown_time) = self.shutdown_time {
            if shutdown_time < self.startup_time {
                return Err(anyhow::anyhow!("Shutdown time before startup time"));
            }
        }

        if self.service_version.is_empty() {
            return Err(anyhow::anyhow!("Empty service version"));
        }

        Ok(())
    }
}

impl Default for ServiceState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_state_manager_file_operations() {
        let temp_dir = tempdir().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        
        let manager = ServiceStateManager::new(state_file).unwrap();
        
        // 创建测试状态
        let mut state = ServiceState::new();
        state.update_timestamp("BTCUSDT", 1640995200000);
        state.add_subscription("tick:BTCUSDT".to_string());
        
        // 保存状态
        manager.save_state(&state).await.unwrap();
        
        // 加载状态
        let loaded_state = manager.load_state().await.unwrap().unwrap();
        
        assert_eq!(loaded_state.startup_time, state.startup_time);
        assert_eq!(loaded_state.last_processed_timestamps, state.last_processed_timestamps);
        assert_eq!(loaded_state.active_subscriptions, state.active_subscriptions);
    }

    #[test]
    fn test_service_state() {
        let mut state = ServiceState::new();
        
        // 测试时间戳更新
        state.update_timestamp("BTCUSDT", 1640995200000);
        assert_eq!(state.get_last_timestamp("BTCUSDT"), Some(1640995200000));
        
        // 测试订阅管理
        state.add_subscription("tick:BTCUSDT".to_string());
        assert_eq!(state.active_subscriptions.len(), 1);
        
        state.remove_subscription("tick:BTCUSDT");
        assert_eq!(state.active_subscriptions.len(), 0);
        
        // 测试消费者偏移量
        state.update_consumer_offset("market_data.ticks", 12345);
        assert_eq!(state.consumer_offsets.get("market_data.ticks"), Some(&12345));
        
        // 测试验证
        assert!(state.validate().is_ok());
    }

    #[test]
    fn test_potential_gaps() {
        let mut state = ServiceState::new();
        
        let now = chrono::Utc::now().timestamp_millis();
        let old_timestamp = now - (10 * 60 * 1000); // 10分钟前
        
        state.update_timestamp("BTCUSDT", old_timestamp);
        state.update_timestamp("ETHUSDT", now);
        
        let gaps = state.has_potential_gaps(5); // 5分钟阈值
        assert_eq!(gaps.len(), 1);
        assert!(gaps.contains(&"BTCUSDT".to_string()));
    }
}