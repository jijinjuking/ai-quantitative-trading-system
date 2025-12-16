use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::CircuitBreakerConfig;

/// 熔断器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // 正常状态
    Open,     // 熔断状态
    HalfOpen, // 半开状态
}

/// 熔断器
#[derive(Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<AtomicU64>,
    half_open_calls: Arc<AtomicU32>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(AtomicU64::new(0)),
            half_open_calls: Arc::new(AtomicU32::new(0)),
        }
    }

    /// 检查是否允许请求通过
    pub async fn allow_request(&self) -> bool {
        let state = self.state.read().await;
        
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // 检查是否可以转换到半开状态
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                let now = Instant::now().elapsed().as_secs();
                
                if now - last_failure >= self.config.recovery_timeout {
                    drop(state);
                    self.transition_to_half_open().await;
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                let current_calls = self.half_open_calls.load(Ordering::Relaxed);
                current_calls < self.config.half_open_max_calls
            }
        }
    }

    /// 记录成功的请求
    pub async fn record_success(&self) {
        let state = self.state.read().await;
        
        match *state {
            CircuitBreakerState::Closed => {
                // 重置失败计数
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.fetch_add(1, Ordering::Relaxed);
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count.fetch_add(1, Ordering::Relaxed);
                let success_count = self.success_count.load(Ordering::Relaxed);
                
                // 如果半开状态下有足够的成功请求，转换到关闭状态
                if success_count >= self.config.half_open_max_calls {
                    drop(state);
                    self.transition_to_closed().await;
                }
            }
            CircuitBreakerState::Open => {
                // 开放状态下不应该有成功请求
                warn!("Received success in open state, this should not happen");
            }
        }
        
        debug!("Circuit breaker recorded success");
    }

    /// 记录失败的请求
    pub async fn record_failure(&self) {
        let state = self.state.read().await;
        
        match *state {
            CircuitBreakerState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                
                // 检查是否达到失败阈值
                if failure_count >= self.config.failure_threshold {
                    drop(state);
                    self.transition_to_open().await;
                }
            }
            CircuitBreakerState::HalfOpen => {
                // 半开状态下的失败立即转换到开放状态
                drop(state);
                self.transition_to_open().await;
            }
            CircuitBreakerState::Open => {
                // 更新最后失败时间
                let now = Instant::now().elapsed().as_secs();
                self.last_failure_time.store(now, Ordering::Relaxed);
            }
        }
        
        debug!("Circuit breaker recorded failure");
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let state = self.get_state().await;
        let failure_count = self.failure_count.load(Ordering::Relaxed);
        let success_count = self.success_count.load(Ordering::Relaxed);
        let half_open_calls = self.half_open_calls.load(Ordering::Relaxed);
        
        CircuitBreakerStats {
            state,
            failure_count,
            success_count,
            half_open_calls,
            failure_threshold: self.config.failure_threshold,
            recovery_timeout: self.config.recovery_timeout,
        }
    }

    /// 转换到关闭状态
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Closed;
        
        // 重置计数器
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_calls.store(0, Ordering::Relaxed);
        
        info!("Circuit breaker transitioned to CLOSED state");
    }

    /// 转换到开放状态
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Open;
        
        // 记录失败时间
        let now = Instant::now().elapsed().as_secs();
        self.last_failure_time.store(now, Ordering::Relaxed);
        
        // 重置半开状态计数器
        self.half_open_calls.store(0, Ordering::Relaxed);
        
        warn!("Circuit breaker transitioned to OPEN state");
    }

    /// 转换到半开状态
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::HalfOpen;
        
        // 重置计数器
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_calls.store(0, Ordering::Relaxed);
        
        info!("Circuit breaker transitioned to HALF_OPEN state");
    }
}

/// 熔断器统计信息
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub half_open_calls: u32,
    pub failure_threshold: u32,
    pub recovery_timeout: u64,
}

/// 熔断器管理器
pub struct CircuitBreakerManager {
    breakers: std::collections::HashMap<String, CircuitBreaker>,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: std::collections::HashMap::new(),
        }
    }

    /// 获取或创建熔断器
    pub fn get_or_create(&mut self, name: &str, config: CircuitBreakerConfig) -> CircuitBreaker {
        self.breakers
            .entry(name.to_string())
            .or_insert_with(|| CircuitBreaker::new(config))
            .clone()
    }

    /// 获取所有熔断器的统计信息
    pub async fn get_all_stats(&self) -> std::collections::HashMap<String, CircuitBreakerStats> {
        let mut stats = std::collections::HashMap::new();
        
        for (name, breaker) in &self.breakers {
            stats.insert(name.clone(), breaker.get_stats().await);
        }
        
        stats
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: 60,
            half_open_max_calls: 2,
        };
        
        let breaker = CircuitBreaker::new(config);
        
        // 初始状态应该是关闭的
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
        assert!(breaker.allow_request().await);
        
        // 记录成功请求
        breaker.record_success().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: 1, // 1秒恢复时间
            half_open_max_calls: 2,
        };
        
        let breaker = CircuitBreaker::new(config);
        
        // 记录失败请求直到达到阈值
        breaker.record_failure().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
        
        breaker.record_failure().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Open);
        
        // 开放状态下不允许请求
        assert!(!breaker.allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: 1,
            half_open_max_calls: 2,
        };
        
        let breaker = CircuitBreaker::new(config);
        
        // 触发开放状态
        breaker.record_failure().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Open);
        
        // 等待恢复时间
        sleep(Duration::from_secs(2)).await;
        
        // 现在应该允许请求（转换到半开状态）
        assert!(breaker.allow_request().await);
        
        // 记录成功请求
        breaker.record_success().await;
        breaker.record_success().await;
        
        // 应该转换回关闭状态
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: 60,
            half_open_max_calls: 2,
        };
        
        let breaker = CircuitBreaker::new(config);
        
        // 记录一些请求
        breaker.record_success().await;
        breaker.record_failure().await;
        
        let stats = breaker.get_stats().await;
        assert_eq!(stats.state, CircuitBreakerState::Closed);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.failure_threshold, 3);
    }
}