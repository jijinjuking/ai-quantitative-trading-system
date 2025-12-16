use anyhow::Result;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::config::RateLimitConfig;

/// 限流器
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    redis: Arc<RwLock<ConnectionManager>>,
}

impl RateLimiter {
    /// 创建新的限流器
    pub fn new(config: RateLimitConfig, redis: Arc<RwLock<ConnectionManager>>) -> Self {
        Self { config, redis }
    }

    /// 检查限流
    pub async fn check_rate_limit(&self, key: &str) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        let full_key = format!("{}rate_limit:{}", 
            self.get_key_prefix(), key);

        match self.check_sliding_window(&full_key).await {
            Ok(allowed) => {
                debug!("Rate limit check for {}: {}", key, if allowed { "allowed" } else { "denied" });
                Ok(allowed)
            }
            Err(e) => {
                warn!("Rate limit check failed for {}: {}", key, e);
                // 失败时允许请求通过（fail-open策略）
                Ok(true)
            }
        }
    }

    /// 滑动窗口限流算法
    async fn check_sliding_window(&self, key: &str) -> Result<bool> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis.write().await;
        let now = chrono::Utc::now().timestamp();
        let window_start = now - self.config.window_size as i64;

        // 使用Redis命令确保原子性
        
        // 移除过期的记录
        let _: () = conn.zrembyscore(key, 0, window_start).await?;
        
        // 添加当前请求
        let _: () = conn.zadd(key, now, now).await?;
        
        // 获取当前窗口内的请求数
        let count: i64 = conn.zcard(key).await?;
        
        // 设置过期时间
        let _: () = conn.expire(key, (self.config.window_size as i64 + 60) as i64).await?;

        let allowed = count <= self.config.requests_per_minute as i64;
        
        debug!("Sliding window check: key={}, count={}, limit={}, allowed={}", 
            key, count, self.config.requests_per_minute, allowed);

        Ok(allowed)
    }

    /// 令牌桶限流算法
    pub async fn check_token_bucket(&self, key: &str) -> Result<bool> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis.write().await;
        let full_key = format!("{}token_bucket:{}", self.get_key_prefix(), key);
        let now = chrono::Utc::now().timestamp();

        // 获取当前令牌桶状态
        let (tokens, last_refill): (Option<i64>, Option<i64>) = redis::pipe()
            .hget(&full_key, "tokens")
            .hget(&full_key, "last_refill")
            .query_async(&mut *conn)
            .await?;

        let mut current_tokens = tokens.unwrap_or(self.config.burst_size as i64);
        let last_refill_time = last_refill.unwrap_or(now);

        // 计算需要补充的令牌数
        let time_passed = now - last_refill_time;
        let tokens_to_add = (time_passed * self.config.requests_per_minute as i64) / 60;
        current_tokens = std::cmp::min(
            current_tokens + tokens_to_add,
            self.config.burst_size as i64,
        );

        let allowed = current_tokens > 0;
        
        if allowed {
            current_tokens -= 1;
        }

        // 更新令牌桶状态
        redis::pipe()
            .atomic()
            .hset(&full_key, "tokens", current_tokens)
            .hset(&full_key, "last_refill", now)
            .expire(&full_key, (self.config.window_size as i64 + 60) as i64)
            .query_async(&mut *conn)
            .await?;

        debug!("Token bucket check: key={}, tokens={}, allowed={}", 
            key, current_tokens, allowed);

        Ok(allowed)
    }

    /// 固定窗口限流算法
    pub async fn check_fixed_window(&self, key: &str) -> Result<bool> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis.write().await;
        let now = chrono::Utc::now().timestamp();
        let window = now / self.config.window_size as i64;
        let full_key = format!("{}fixed_window:{}:{}", 
            self.get_key_prefix(), key, window);

        // 原子性地增加计数并获取当前值
        let count: i64 = conn.incr(&full_key, 1).await?;
        
        // 设置过期时间（仅在第一次设置时）
        if count == 1 {
            let _: () = conn.expire(&full_key, self.config.window_size as i64).await?;
        }

        let allowed = count <= self.config.requests_per_minute as i64;
        
        debug!("Fixed window check: key={}, window={}, count={}, allowed={}", 
            key, window, count, allowed);

        Ok(allowed)
    }

    /// 获取限流统计信息
    pub async fn get_rate_limit_info(&self, key: &str) -> Result<RateLimitInfo> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis.write().await;
        let full_key = format!("{}rate_limit:{}", self.get_key_prefix(), key);
        let now = chrono::Utc::now().timestamp();
        let window_start = now - self.config.window_size as i64;

        // 获取当前窗口内的请求数
        let count: i64 = conn.zcount(&full_key, window_start, now).await?;
        
        let remaining = std::cmp::max(0, self.config.requests_per_minute as i64 - count);
        let reset_time = now + self.config.window_size as i64;

        Ok(RateLimitInfo {
            limit: self.config.requests_per_minute,
            remaining: remaining as u32,
            reset_time,
            window_size: self.config.window_size,
        })
    }

    /// 重置限流计数
    pub async fn reset_rate_limit(&self, key: &str) -> Result<()> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis.write().await;
        let full_key = format!("{}rate_limit:{}", self.get_key_prefix(), key);
        
        let _: () = conn.del(&full_key).await?;
        
        debug!("Rate limit reset for key: {}", key);
        Ok(())
    }

    /// 批量检查限流
    pub async fn check_batch_rate_limit(&self, keys: &[String]) -> Result<Vec<bool>> {
        let mut results = Vec::with_capacity(keys.len());
        
        for key in keys {
            let allowed = self.check_rate_limit(key).await?;
            results.push(allowed);
        }
        
        Ok(results)
    }

    /// 获取键前缀
    fn get_key_prefix(&self) -> &str {
        "gateway:"
    }

    /// 清理过期的限流记录
    pub async fn cleanup_expired_records(&self) -> Result<u64> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis.write().await;
        let pattern = format!("{}rate_limit:*", self.get_key_prefix());
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - self.config.window_size as i64 * 2; // 保留2个窗口的数据
        
        let keys: Vec<String> = conn.keys(&pattern).await?;
        let mut cleaned = 0u64;
        
        for key in keys {
            let removed: i64 = conn.zrembyscore(&key, 0, cutoff).await?;
            cleaned += removed as u64;
            
            // 如果集合为空，删除整个键
            let count: i64 = conn.zcard(&key).await?;
            if count == 0 {
                let _: () = conn.del(&key).await?;
            }
        }
        
        debug!("Cleaned {} expired rate limit records", cleaned);
        Ok(cleaned)
    }
}

/// 限流信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_time: i64,
    pub window_size: u64,
}

/// 限流策略
#[derive(Debug, Clone)]
pub enum RateLimitStrategy {
    SlidingWindow,
    TokenBucket,
    FixedWindow,
}

/// 限流器构建器
pub struct RateLimiterBuilder {
    config: RateLimitConfig,
    strategy: RateLimitStrategy,
}

impl RateLimiterBuilder {
    pub fn new() -> Self {
        Self {
            config: RateLimitConfig::default(),
            strategy: RateLimitStrategy::SlidingWindow,
        }
    }

    pub fn requests_per_minute(mut self, requests: u32) -> Self {
        self.config.requests_per_minute = requests;
        self
    }

    pub fn burst_size(mut self, size: u32) -> Self {
        self.config.burst_size = size;
        self
    }

    pub fn window_size(mut self, size: u64) -> Self {
        self.config.window_size = size;
        self
    }

    pub fn strategy(mut self, strategy: RateLimitStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn whitelist(mut self, ips: Vec<String>) -> Self {
        self.config.whitelist = ips;
        self
    }

    pub fn build(self, redis: Arc<RwLock<ConnectionManager>>) -> RateLimiter {
        RateLimiter::new(self.config, redis)
    }
}

impl Default for RateLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_info() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset_time: 1640995200,
            window_size: 60,
        };

        assert_eq!(info.limit, 100);
        assert_eq!(info.remaining, 50);
        assert_eq!(info.reset_time, 1640995200);
        assert_eq!(info.window_size, 60);
    }

    #[test]
    fn test_rate_limiter_builder() {
        let config = RateLimitConfig::default();
        let builder = RateLimiterBuilder::new()
            .requests_per_minute(200)
            .burst_size(50)
            .window_size(120)
            .strategy(RateLimitStrategy::TokenBucket)
            .whitelist(vec!["127.0.0.1".to_string()]);

        assert_eq!(builder.config.requests_per_minute, 200);
        assert_eq!(builder.config.burst_size, 50);
        assert_eq!(builder.config.window_size, 120);
        assert!(matches!(builder.strategy, RateLimitStrategy::TokenBucket));
    }
}