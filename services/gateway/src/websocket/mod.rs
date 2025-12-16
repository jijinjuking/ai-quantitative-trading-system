pub mod proxy;
pub mod connection;
pub mod message;
pub mod pool;

pub use proxy::{WebSocketProxy, ProxyStats};
pub use connection::{WebSocketConnection, ConnectionState};
pub use message::{WebSocketMessage, MessageType};
// 暂时注释pool导出，使用proxy中的简化版本
// pub use pool::{ConnectionPool, PoolStats, PoolHealthStatus};
pub use proxy::ConnectionPool;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// WebSocket代理管理器
#[derive(Clone)]
pub struct WebSocketManager {
    connection_pool: Arc<ConnectionPool>,
    proxy: Arc<WebSocketProxy>,
}

impl WebSocketManager {
    /// 创建新的WebSocket管理器
    pub fn new() -> Self {
        let connection_pool = Arc::new(ConnectionPool::new());
        let proxy = Arc::new(WebSocketProxy::new(connection_pool.clone()));

        Self {
            connection_pool,
            proxy,
        }
    }

    /// 处理WebSocket连接
    pub async fn handle_connection(
        &self,
        ws_upgrade: axum::extract::ws::WebSocketUpgrade,
        service_name: &str,
        target_url: &str,
    ) -> Result<axum::response::Response> {
        info!("Handling WebSocket connection for service: {}", service_name);
        
        self.proxy.proxy_connection(ws_upgrade, service_name, target_url).await
    }

    /// 获取连接池统计
    pub async fn get_pool_stats(&self) -> ProxyStats {
        self.connection_pool.get_stats().await
    }

    /// 关闭所有连接
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down WebSocket manager");
        // 简化实现，暂时返回Ok
        Ok(())
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}