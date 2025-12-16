use anyhow::Result;
use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    http::HeaderMap,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::{
    connection::{WebSocketConnection, ConnectionConfig},
    message::WebSocketMessage,
};

// 临时简化的ConnectionPool定义，避免循环依赖
#[derive(Clone)]
pub struct ConnectionPool {
    // 简化实现，仅用于编译通过
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn can_accept_connection(&self, _service_name: &str) -> bool {
        true // 简化实现
    }
    
    pub async fn add_connection(&self, _connection: WebSocketConnection) {
        // 简化实现
    }
    
    pub async fn remove_connection(&self, _connection_id: &str) {
        // 简化实现
    }
    
    pub async fn get_stats(&self) -> ProxyStats {
        ProxyStats {
            total_connections: 0,
            active_connections: 0,
            connections_by_service: std::collections::HashMap::new(),
            total_messages: 0,
            total_errors: 0,
        }
    }
    
    pub async fn close_service_connections(&self, _service_name: &str) -> anyhow::Result<usize> {
        Ok(0)
    }
    
    pub async fn broadcast_to_service(&self, _service_name: &str, _message: WebSocketMessage) -> anyhow::Result<usize> {
        Ok(0)
    }
    
    pub async fn send_to_connection(&self, _connection_id: &str, _message: WebSocketMessage) -> anyhow::Result<()> {
        Ok(())
    }
    
    pub async fn get_connection_stats(&self, _connection_id: &str) -> Option<super::connection::ConnectionStats> {
        None
    }
}

/// WebSocket代理服务
#[derive(Clone)]
pub struct WebSocketProxy {
    connection_pool: Arc<ConnectionPool>,
    config: ConnectionConfig,
}

impl WebSocketProxy {
    /// 创建新的WebSocket代理
    pub fn new(connection_pool: Arc<ConnectionPool>) -> Self {
        Self {
            connection_pool,
            config: ConnectionConfig::default(),
        }
    }

    /// 使用自定义配置创建WebSocket代理
    pub fn with_config(connection_pool: Arc<ConnectionPool>, config: ConnectionConfig) -> Self {
        Self {
            connection_pool,
            config,
        }
    }

    /// 代理WebSocket连接
    pub async fn proxy_connection(
        &self,
        ws_upgrade: WebSocketUpgrade,
        service_name: &str,
        target_url: &str,
    ) -> Result<Response> {
        info!("Creating WebSocket proxy for service: {} -> {}", service_name, target_url);

        // 验证目标URL
        if !self.is_valid_websocket_url(target_url) {
            error!("Invalid WebSocket target URL: {}", target_url);
            return Err(anyhow::anyhow!("Invalid WebSocket URL"));
        }

        // 检查连接池限制
        if !self.connection_pool.can_accept_connection(service_name).await {
            warn!("Connection pool limit reached for service: {}", service_name);
            return Err(anyhow::anyhow!("Connection pool limit reached"));
        }

        // 创建WebSocket连接对象
        let connection = WebSocketConnection::new(
            service_name.to_string(),
            target_url.to_string(),
        );

        let connection_id = connection.id.clone();
        let pool = self.connection_pool.clone();

        // 将连接添加到池中
        pool.add_connection(connection.clone()).await;

        // 创建WebSocket升级响应
        let response = ws_upgrade.on_upgrade(move |socket| async move {
            if let Err(e) = Self::handle_websocket_connection(socket, connection, pool).await {
                error!("WebSocket connection error: {}", e);
            }
        });

        Ok(response)
    }

    /// 处理WebSocket连接
    async fn handle_websocket_connection(
        socket: WebSocket,
        connection: WebSocketConnection,
        pool: Arc<ConnectionPool>,
    ) -> Result<()> {
        let connection_id = connection.id.clone();
        
        info!("Handling WebSocket connection: {}", connection_id);

        // 建立代理连接
        let result = connection.establish_proxy(socket).await;

        // 从连接池中移除连接
        pool.remove_connection(&connection_id).await;

        match &result {
            Ok(_) => {
                info!("WebSocket proxy completed successfully: {}", connection_id);
            }
            Err(e) => {
                error!("WebSocket proxy failed: {} - {}", connection_id, e);
            }
        }

        result
    }

    /// 验证WebSocket URL
    fn is_valid_websocket_url(&self, url: &str) -> bool {
        url.starts_with("ws://") || url.starts_with("wss://")
    }

    /// 构建WebSocket目标URL
    pub fn build_target_url(&self, base_url: &str, path: &str) -> String {
        let ws_url = if base_url.starts_with("http://") {
            base_url.replace("http://", "ws://")
        } else if base_url.starts_with("https://") {
            base_url.replace("https://", "wss://")
        } else {
            format!("ws://{}", base_url)
        };

        format!("{}{}", ws_url.trim_end_matches('/'), path)
    }

    /// 获取代理统计信息
    pub async fn get_proxy_stats(&self) -> ProxyStats {
        let pool_stats = self.connection_pool.get_stats().await;
        
        ProxyStats {
            total_connections: pool_stats.total_connections,
            active_connections: pool_stats.active_connections,
            connections_by_service: pool_stats.connections_by_service,
            total_messages: pool_stats.total_messages,
            total_errors: pool_stats.total_errors,
        }
    }

    /// 关闭指定服务的所有连接
    pub async fn close_service_connections(&self, service_name: &str) -> Result<usize> {
        info!("Closing all connections for service: {}", service_name);
        let closed_count = self.connection_pool.close_service_connections(service_name).await?;
        info!("Closed {} connections for service: {}", closed_count, service_name);
        Ok(closed_count)
    }

    /// 广播消息到指定服务的所有连接
    pub async fn broadcast_to_service(
        &self,
        service_name: &str,
        message: WebSocketMessage,
    ) -> Result<usize> {
        debug!("Broadcasting message to service: {}", service_name);
        let sent_count = self.connection_pool.broadcast_to_service(service_name, message).await?;
        debug!("Broadcasted message to {} connections", sent_count);
        Ok(sent_count)
    }

    /// 发送消息到特定连接
    pub async fn send_to_connection(
        &self,
        connection_id: &str,
        message: WebSocketMessage,
    ) -> Result<()> {
        debug!("Sending message to connection: {}", connection_id);
        self.connection_pool.send_to_connection(connection_id, message).await
    }

    /// 获取连接详细信息
    pub async fn get_connection_info(&self, connection_id: &str) -> Option<super::connection::ConnectionStats> {
        self.connection_pool.get_connection_stats(connection_id).await
    }

    /// 健康检查
    pub async fn health_check(&self) -> ProxyHealthStatus {
        let pool_stats = self.connection_pool.get_stats().await;
        let is_healthy = pool_stats.total_errors < pool_stats.total_messages / 10; // 错误率小于10%

        ProxyHealthStatus {
            is_healthy,
            active_connections: pool_stats.active_connections,
            total_connections: pool_stats.total_connections,
            error_rate: if pool_stats.total_messages > 0 {
                pool_stats.total_errors as f64 / pool_stats.total_messages as f64
            } else {
                0.0
            },
        }
    }
}

/// 代理统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProxyStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub connections_by_service: std::collections::HashMap<String, usize>,
    pub total_messages: u64,
    pub total_errors: u64,
}

/// 代理健康状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProxyHealthStatus {
    pub is_healthy: bool,
    pub active_connections: usize,
    pub total_connections: usize,
    pub error_rate: f64,
}

/// WebSocket代理配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WebSocketProxyConfig {
    pub enabled: bool,
    pub max_connections: usize,
    pub max_connections_per_service: usize,
    pub connection_timeout: std::time::Duration,
    pub message_timeout: std::time::Duration,
    pub heartbeat_interval: std::time::Duration,
    pub max_message_size: usize,
    pub allowed_origins: Vec<String>,
    pub rate_limit: WebSocketRateLimit,
}

/// WebSocket限流配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WebSocketRateLimit {
    pub enabled: bool,
    pub messages_per_second: u32,
    pub burst_size: u32,
    pub window_size: std::time::Duration,
}

impl Default for WebSocketProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_connections: 10000,
            max_connections_per_service: 1000,
            connection_timeout: std::time::Duration::from_secs(30),
            message_timeout: std::time::Duration::from_secs(10),
            heartbeat_interval: std::time::Duration::from_secs(30),
            max_message_size: 1024 * 1024, // 1MB
            allowed_origins: vec!["*".to_string()],
            rate_limit: WebSocketRateLimit {
                enabled: true,
                messages_per_second: 100,
                burst_size: 200,
                window_size: std::time::Duration::from_secs(60),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_target_url() {
        let pool = Arc::new(ConnectionPool::new());
        let proxy = WebSocketProxy::new(pool);

        // HTTP to WebSocket
        let url = proxy.build_target_url("http://localhost:8080", "/ws/market");
        assert_eq!(url, "ws://localhost:8080/ws/market");

        // HTTPS to WebSocket Secure
        let url = proxy.build_target_url("https://api.example.com", "/ws/trading");
        assert_eq!(url, "wss://api.example.com/ws/trading");

        // Already WebSocket URL
        let url = proxy.build_target_url("ws://localhost:9000", "/stream");
        assert_eq!(url, "ws://localhost:9000/stream");
    }

    #[test]
    fn test_is_valid_websocket_url() {
        let pool = Arc::new(ConnectionPool::new());
        let proxy = WebSocketProxy::new(pool);

        assert!(proxy.is_valid_websocket_url("ws://localhost:8080"));
        assert!(proxy.is_valid_websocket_url("wss://api.example.com"));
        assert!(!proxy.is_valid_websocket_url("http://localhost:8080"));
        assert!(!proxy.is_valid_websocket_url("invalid-url"));
    }
}