use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{ConnectionStats, ConnectorError};

/// 连接池管理器
#[derive(Clone)]
pub struct ConnectionPool {
    connections: Arc<RwLock<HashMap<String, PooledConnection>>>,
    config: ConnectionPoolConfig,
    stats: Arc<RwLock<ConnectionPoolStats>>,
}

/// 连接池配置
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub max_idle_time: Duration,
    pub health_check_interval: Duration,
    pub connection_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            max_idle_time: Duration::from_secs(300), // 5分钟
            health_check_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
        }
    }
}

/// 池化连接
#[derive(Debug, Clone)]
pub struct PooledConnection {
    pub id: String,
    pub exchange: String,
    pub url: String,
    pub status: ConnectionStatus,
    pub created_at: Instant,
    pub last_used: Instant,
    pub last_health_check: Option<Instant>,
    pub stats: ConnectionStats,
    pub retry_count: u32,
}

/// 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Idle,
    Active,
    Connecting,
    Disconnected,
    Error(String),
}

/// 连接池统计信息
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub error_connections: usize,
    pub connections_created: u64,
    pub connections_destroyed: u64,
    pub health_checks_performed: u64,
    pub health_checks_failed: u64,
}

impl ConnectionPool {
    /// 创建新的连接池
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let pool = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(ConnectionPoolStats::default())),
        };

        // 启动健康检查任务
        pool.start_health_check_task();

        pool
    }

    /// 获取连接
    pub async fn get_connection(&self, exchange: &str, url: &str) -> Result<PooledConnection> {
        let connection_key = format!("{}:{}", exchange, url);
        
        // 首先尝试获取现有连接
        {
            let mut connections = self.connections.write().await;
            
            if let Some(connection) = connections.get_mut(&connection_key) {
                if connection.status == ConnectionStatus::Idle {
                    connection.status = ConnectionStatus::Active;
                    connection.last_used = Instant::now();
                    debug!("Reusing existing connection: {}", connection_key);
                    return Ok(connection.clone());
                }
            }
        }

        // 检查连接池是否已满
        {
            let connections = self.connections.read().await;
            if connections.len() >= self.config.max_connections {
                return Err(ConnectorError::ConnectionFailed(
                    "Connection pool is full".to_string()
                ).into());
            }
        }

        // 创建新连接
        self.create_connection(exchange, url).await
    }

    /// 创建新连接
    async fn create_connection(&self, exchange: &str, url: &str) -> Result<PooledConnection> {
        let connection_key = format!("{}:{}", exchange, url);
        let connection_id = uuid::Uuid::new_v4().to_string();
        
        info!("Creating new connection: {} ({})", connection_key, connection_id);

        let connection = PooledConnection {
            id: connection_id,
            exchange: exchange.to_string(),
            url: url.to_string(),
            status: ConnectionStatus::Connecting,
            created_at: Instant::now(),
            last_used: Instant::now(),
            last_health_check: None,
            stats: ConnectionStats::default(),
            retry_count: 0,
        };

        // 添加到连接池
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_key, connection.clone());
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_connections += 1;
            stats.connections_created += 1;
        }

        Ok(connection)
    }

    /// 释放连接
    pub async fn release_connection(&self, connection: &PooledConnection) -> Result<()> {
        let connection_key = format!("{}:{}", connection.exchange, connection.url);
        
        let mut connections = self.connections.write().await;
        
        if let Some(pooled_connection) = connections.get_mut(&connection_key) {
            pooled_connection.status = ConnectionStatus::Idle;
            pooled_connection.last_used = Instant::now();
            debug!("Connection released: {}", connection_key);
        }

        Ok(())
    }

    /// 移除连接
    pub async fn remove_connection(&self, connection: &PooledConnection) -> Result<()> {
        let connection_key = format!("{}:{}", connection.exchange, connection.url);
        
        let mut connections = self.connections.write().await;
        
        if connections.remove(&connection_key).is_some() {
            info!("Connection removed: {}", connection_key);
            
            // 更新统计信息
            let mut stats = self.stats.write().await;
            stats.total_connections = stats.total_connections.saturating_sub(1);
            stats.connections_destroyed += 1;
        }

        Ok(())
    }

    /// 标记连接为错误状态
    pub async fn mark_connection_error(&self, connection: &PooledConnection, error: String) -> Result<()> {
        let connection_key = format!("{}:{}", connection.exchange, connection.url);
        
        let mut connections = self.connections.write().await;
        
        if let Some(pooled_connection) = connections.get_mut(&connection_key) {
            pooled_connection.status = ConnectionStatus::Error(error.clone());
            pooled_connection.retry_count += 1;
            warn!("Connection marked as error: {} - {}", connection_key, error);
        }

        Ok(())
    }

    /// 获取所有连接
    pub async fn get_all_connections(&self) -> HashMap<String, PooledConnection> {
        let connections = self.connections.read().await;
        connections.clone()
    }

    /// 获取指定交易所的连接
    pub async fn get_exchange_connections(&self, exchange: &str) -> Vec<PooledConnection> {
        let connections = self.connections.read().await;
        
        connections
            .values()
            .filter(|conn| conn.exchange == exchange)
            .cloned()
            .collect()
    }

    /// 清理空闲连接
    pub async fn cleanup_idle_connections(&self) -> Result<usize> {
        let mut removed_count = 0;
        let now = Instant::now();
        
        let mut connections = self.connections.write().await;
        let mut to_remove = Vec::new();
        
        for (key, connection) in connections.iter() {
            if connection.status == ConnectionStatus::Idle {
                let idle_time = now.duration_since(connection.last_used);
                if idle_time > self.config.max_idle_time {
                    to_remove.push(key.clone());
                }
            }
        }
        
        for key in to_remove {
            if connections.remove(&key).is_some() {
                removed_count += 1;
                debug!("Removed idle connection: {}", key);
            }
        }
        
        if removed_count > 0 {
            info!("Cleaned up {} idle connections", removed_count);
            
            // 更新统计信息
            let mut stats = self.stats.write().await;
            stats.total_connections = connections.len();
            stats.connections_destroyed += removed_count as u64;
        }
        
        Ok(removed_count)
    }

    /// 启动健康检查任务
    fn start_health_check_task(&self) {
        let pool = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(pool.config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = pool.perform_health_checks().await {
                    error!("Health check failed: {}", e);
                }
                
                if let Err(e) = pool.cleanup_idle_connections().await {
                    error!("Idle connection cleanup failed: {}", e);
                }
            }
        });
        
        info!("Connection pool health check task started");
    }

    /// 执行健康检查
    async fn perform_health_checks(&self) -> Result<()> {
        let connections = self.connections.read().await;
        let mut health_check_count = 0;
        let mut failed_count = 0;
        
        for (key, connection) in connections.iter() {
            // 跳过正在连接或已断开的连接
            if matches!(connection.status, ConnectionStatus::Connecting | ConnectionStatus::Disconnected) {
                continue;
            }
            
            health_check_count += 1;
            
            // 这里应该实际执行健康检查，比如发送ping消息
            // 为了简化，我们只是检查连接的最后使用时间
            let now = Instant::now();
            let last_activity = connection.last_used;
            
            if now.duration_since(last_activity) > Duration::from_secs(300) {
                // 5分钟没有活动，可能需要检查
                debug!("Connection {} may need health check", key);
            }
        }
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.health_checks_performed += health_check_count;
            stats.health_checks_failed += failed_count;
        }
        
        debug!("Health checks completed: {} checked, {} failed", health_check_count, failed_count);
        Ok(())
    }

    /// 获取连接池统计信息
    pub async fn get_stats(&self) -> ConnectionPoolStats {
        let connections = self.connections.read().await;
        let mut stats = self.stats.read().await.clone();
        
        // 更新实时统计
        stats.total_connections = connections.len();
        stats.active_connections = connections.values()
            .filter(|conn| conn.status == ConnectionStatus::Active)
            .count();
        stats.idle_connections = connections.values()
            .filter(|conn| conn.status == ConnectionStatus::Idle)
            .count();
        stats.error_connections = connections.values()
            .filter(|conn| matches!(conn.status, ConnectionStatus::Error(_)))
            .count();
        
        stats
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ConnectionPoolStats::default();
        info!("Connection pool statistics reset");
    }

    /// 关闭连接池
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down connection pool");
        
        let mut connections = self.connections.write().await;
        let connection_count = connections.len();
        
        connections.clear();
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_connections = 0;
            stats.active_connections = 0;
            stats.idle_connections = 0;
            stats.error_connections = 0;
            stats.connections_destroyed += connection_count as u64;
        }
        
        info!("Connection pool shutdown completed, {} connections closed", connection_count);
        Ok(())
    }

    /// 获取连接池健康状态
    pub async fn health_status(&self) -> ConnectionPoolHealth {
        let stats = self.get_stats().await;
        let connections = self.connections.read().await;
        
        let healthy_connections = connections.values()
            .filter(|conn| matches!(conn.status, ConnectionStatus::Active | ConnectionStatus::Idle))
            .count();
        
        let health_score = if stats.total_connections > 0 {
            healthy_connections as f64 / stats.total_connections as f64
        } else {
            1.0
        };
        
        ConnectionPoolHealth {
            is_healthy: health_score >= 0.8, // 80%的连接健康
            total_connections: stats.total_connections,
            healthy_connections,
            error_connections: stats.error_connections,
            health_score,
            pool_utilization: stats.total_connections as f64 / self.config.max_connections as f64,
        }
    }
}

/// 连接池健康状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionPoolHealth {
    pub is_healthy: bool,
    pub total_connections: usize,
    pub healthy_connections: usize,
    pub error_connections: usize,
    pub health_score: f64,
    pub pool_utilization: f64,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(ConnectionPoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_creation_and_release() {
        let pool = ConnectionPool::default();
        
        // 创建连接
        let connection = pool.get_connection("binance", "wss://stream.binance.com/ws").await.unwrap();
        assert_eq!(connection.exchange, "binance");
        assert_eq!(connection.status, ConnectionStatus::Connecting);
        
        // 释放连接
        pool.release_connection(&connection).await.unwrap();
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 1);
    }

    #[tokio::test]
    async fn test_connection_removal() {
        let pool = ConnectionPool::default();
        
        let connection = pool.get_connection("binance", "wss://stream.binance.com/ws").await.unwrap();
        pool.remove_connection(&connection).await.unwrap();
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.connections_destroyed, 1);
    }

    #[tokio::test]
    async fn test_health_status() {
        let pool = ConnectionPool::default();
        
        let health = pool.health_status().await;
        assert!(health.is_healthy); // 空池被认为是健康的
        assert_eq!(health.total_connections, 0);
        assert_eq!(health.pool_utilization, 0.0);
    }
}