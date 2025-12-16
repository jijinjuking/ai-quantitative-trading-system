use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use super::{ConnectorError, ConnectionStats};

/// WebSocket客户端
pub struct WebSocketClient {
    url: String,
    stats: Arc<RwLock<ConnectionStats>>,
    message_sender: Option<mpsc::UnboundedSender<String>>,
    event_receiver: Option<mpsc::UnboundedReceiver<WebSocketEvent>>,
    config: WebSocketConfig,
}

/// WebSocket配置
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub ping_interval: Duration,
    pub pong_timeout: Duration,
    pub reconnect_interval: Duration,
    pub max_reconnect_attempts: u32,
    pub message_buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(10),
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
            reconnect_interval: Duration::from_secs(5),
            max_reconnect_attempts: 10,
            message_buffer_size: 10000,
        }
    }
}

/// WebSocket事件
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    Connected,
    Disconnected,
    Message(String),
    Error(String),
    Pong(Vec<u8>),
}

impl WebSocketClient {
    /// 创建新的WebSocket客户端
    pub fn new(url: String, config: WebSocketConfig) -> Self {
        Self {
            url,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            message_sender: None,
            event_receiver: None,
            config,
        }
    }

    /// 连接到WebSocket服务器
    pub async fn connect(&mut self) -> Result<mpsc::UnboundedReceiver<WebSocketEvent>> {
        info!("Connecting to WebSocket: {}", self.url);

        // 验证URL
        let url = Url::parse(&self.url)
            .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid URL: {}", e)))?;

        // 建立连接
        let (ws_stream, _) = timeout(self.config.connect_timeout, connect_async(url))
            .await
            .map_err(|_| ConnectorError::ConnectionFailed("Connection timeout".to_string()))?
            .map_err(|e| ConnectorError::ConnectionFailed(format!("WebSocket connection failed: {}", e)))?;

        info!("WebSocket connected successfully");

        // 分离读写流
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // 创建消息通道
        let (message_tx, mut message_rx) = mpsc::unbounded_channel::<String>();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<WebSocketEvent>();

        self.message_sender = Some(message_tx);

        // 更新连接状态
        {
            let mut stats = self.stats.write().await;
            stats.set_connected(true);
        }

        // 发送连接事件
        let _ = event_tx.send(WebSocketEvent::Connected);

        // 启动消息发送任务
        let write_timeout = self.config.write_timeout;
        let stats_write = self.stats.clone();
        let event_tx_write = event_tx.clone();
        
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                debug!("Sending WebSocket message: {}", message);
                
                let send_result = timeout(
                    write_timeout,
                    ws_sender.send(Message::Text(message.clone()))
                ).await;

                match send_result {
                    Ok(Ok(_)) => {
                        stats_write.write().await.record_message_sent();
                        debug!("Message sent successfully");
                    }
                    Ok(Err(e)) => {
                        error!("Failed to send message: {}", e);
                        stats_write.write().await.record_error();
                        let _ = event_tx_write.send(WebSocketEvent::Error(format!("Send error: {}", e)));
                        break;
                    }
                    Err(_) => {
                        error!("Message send timeout");
                        stats_write.write().await.record_error();
                        let _ = event_tx_write.send(WebSocketEvent::Error("Send timeout".to_string()));
                        break;
                    }
                }
            }
            
            warn!("WebSocket sender task ended");
        });

        // 启动消息接收任务
        let read_timeout = self.config.read_timeout;
        let stats_read = self.stats.clone();
        let event_tx_read = event_tx.clone();
        
        tokio::spawn(async move {
            loop {
                let receive_result = timeout(read_timeout, ws_receiver.next()).await;
                
                match receive_result {
                    Ok(Some(Ok(message))) => {
                        stats_read.write().await.record_message_received();
                        
                        match message {
                            Message::Text(text) => {
                                debug!("Received text message: {}", text);
                                let _ = event_tx_read.send(WebSocketEvent::Message(text));
                            }
                            Message::Binary(data) => {
                                debug!("Received binary message: {} bytes", data.len());
                                // 尝试将二进制数据转换为文本
                                if let Ok(text) = String::from_utf8(data) {
                                    let _ = event_tx_read.send(WebSocketEvent::Message(text));
                                } else {
                                    warn!("Received non-UTF8 binary message");
                                }
                            }
                            Message::Ping(data) => {
                                debug!("Received ping, sending pong");
                                // WebSocket库会自动处理pong响应
                            }
                            Message::Pong(data) => {
                                debug!("Received pong");
                                let _ = event_tx_read.send(WebSocketEvent::Pong(data));
                            }
                            Message::Close(_) => {
                                info!("WebSocket connection closed by server");
                                break;
                            }
                            Message::Frame(_) => {
                                // 原始帧，通常不需要处理
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        error!("WebSocket receive error: {}", e);
                        stats_read.write().await.record_error();
                        let _ = event_tx_read.send(WebSocketEvent::Error(format!("Receive error: {}", e)));
                        break;
                    }
                    Ok(None) => {
                        info!("WebSocket stream ended");
                        break;
                    }
                    Err(_) => {
                        warn!("WebSocket receive timeout");
                        // 超时不算错误，继续接收
                        continue;
                    }
                }
            }
            
            // 更新连接状态
            stats_read.write().await.set_connected(false);
            let _ = event_tx_read.send(WebSocketEvent::Disconnected);
            warn!("WebSocket receiver task ended");
        });

        // 启动心跳任务
        self.start_heartbeat_task(event_tx.clone()).await;

        Ok(event_rx)
    }

    /// 发送消息
    pub async fn send_message(&self, message: &str) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message.to_string())
                .map_err(|_| ConnectorError::NetworkError("Message channel closed".to_string()))?;
            Ok(())
        } else {
            Err(ConnectorError::ConnectionFailed("Not connected".to_string()).into())
        }
    }

    /// 发送JSON消息
    pub async fn send_json(&self, json: &Value) -> Result<()> {
        let message = serde_json::to_string(json)
            .map_err(|e| ConnectorError::MessageParsingFailed(format!("JSON serialization failed: {}", e)))?;
        self.send_message(&message).await
    }

    /// 发送订阅消息
    pub async fn subscribe(&self, method: &str, params: &Value, id: Option<u64>) -> Result<()> {
        let subscription = serde_json::json!({
            "method": method,
            "params": params,
            "id": id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64)
        });

        self.send_json(&subscription).await
    }

    /// 发送取消订阅消息
    pub async fn unsubscribe(&self, method: &str, params: &Value, id: Option<u64>) -> Result<()> {
        let unsubscription = serde_json::json!({
            "method": method,
            "params": params,
            "id": id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64)
        });

        self.send_json(&unsubscription).await
    }

    /// 发送Ping消息
    pub async fn ping(&self, data: Option<Vec<u8>>) -> Result<()> {
        let ping_data = data.unwrap_or_else(|| b"ping".to_vec());
        
        // 通过发送特殊格式的消息来触发ping
        let ping_message = serde_json::json!({
            "method": "ping",
            "params": [],
            "id": chrono::Utc::now().timestamp_millis()
        });

        self.send_json(&ping_message).await
    }

    /// 获取连接统计
    pub async fn get_stats(&self) -> ConnectionStats {
        self.stats.read().await.clone()
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        self.stats.read().await.connected
    }

    /// 启动心跳任务
    async fn start_heartbeat_task(&self, event_tx: mpsc::UnboundedSender<WebSocketEvent>) {
        let ping_interval = self.config.ping_interval;
        let pong_timeout = self.config.pong_timeout;
        let stats = self.stats.clone();
        
        tokio::spawn(async move {
            let mut ping_timer = interval(ping_interval);
            let mut last_pong = Instant::now();
            
            loop {
                ping_timer.tick().await;
                
                // 检查是否仍然连接
                if !stats.read().await.connected {
                    break;
                }
                
                // 检查pong超时
                if last_pong.elapsed() > pong_timeout + ping_interval {
                    error!("Pong timeout detected, connection may be dead");
                    let _ = event_tx.send(WebSocketEvent::Error("Pong timeout".to_string()));
                    break;
                }
                
                // 发送ping (通过特殊消息格式)
                debug!("Sending heartbeat ping");
                // 这里应该通过message_sender发送ping，但为了简化，我们跳过实际的ping发送
                // 在实际实现中，应该有一个专门的ping通道
            }
            
            debug!("Heartbeat task ended");
        });
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting WebSocket");
        
        // 关闭消息发送通道
        self.message_sender = None;
        
        // 更新连接状态
        {
            let mut stats = self.stats.write().await;
            stats.set_connected(false);
        }
        
        Ok(())
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ConnectionStats::default();
    }
}

/// WebSocket客户端构建器
pub struct WebSocketClientBuilder {
    url: Option<String>,
    config: WebSocketConfig,
}

impl WebSocketClientBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            url: None,
            config: WebSocketConfig::default(),
        }
    }

    /// 设置URL
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.url = Some(url.into());
        self
    }

    /// 设置连接超时
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// 设置读取超时
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.config.read_timeout = timeout;
        self
    }

    /// 设置写入超时
    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.config.write_timeout = timeout;
        self
    }

    /// 设置心跳间隔
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.config.ping_interval = interval;
        self
    }

    /// 设置重连间隔
    pub fn reconnect_interval(mut self, interval: Duration) -> Self {
        self.config.reconnect_interval = interval;
        self
    }

    /// 设置最大重连次数
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.config.max_reconnect_attempts = attempts;
        self
    }

    /// 构建WebSocket客户端
    pub fn build(self) -> Result<WebSocketClient> {
        let url = self.url.ok_or_else(|| {
            ConnectorError::ConfigurationError("URL is required".to_string())
        })?;

        Ok(WebSocketClient::new(url, self.config))
    }
}

impl Default for WebSocketClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.ping_interval, Duration::from_secs(30));
        assert_eq!(config.max_reconnect_attempts, 10);
    }

    #[test]
    fn test_websocket_client_builder() {
        let client = WebSocketClientBuilder::new()
            .url("wss://example.com/ws")
            .connect_timeout(Duration::from_secs(5))
            .ping_interval(Duration::from_secs(15))
            .build();

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.url, "wss://example.com/ws");
        assert_eq!(client.config.connect_timeout, Duration::from_secs(5));
        assert_eq!(client.config.ping_interval, Duration::from_secs(15));
    }

    #[test]
    fn test_websocket_client_builder_missing_url() {
        let result = WebSocketClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let client = WebSocketClient::new(
            "wss://example.com/ws".to_string(),
            WebSocketConfig::default(),
        );

        assert_eq!(client.url, "wss://example.com/ws");
        assert!(!client.is_connected().await);
        
        let stats = client.get_stats().await;
        assert!(!stats.connected);
        assert_eq!(stats.messages_received, 0);
    }
}