use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use shared_models::*;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    // 认证相关
    Auth { token: String },
    AuthSuccess { user_id: String },
    AuthError { message: String },

    // 订阅相关
    Subscribe { channels: Vec<String> },
    Unsubscribe { channels: Vec<String> },
    SubscribeSuccess { channels: Vec<String> },
    SubscribeError { message: String },

    // 市场数据
    MarketTick(MarketTick),
    Kline(Kline),
    OrderBook(OrderBook),
    Ticker24hr(Ticker24hr),
    Trade(shared_models::market::Trade),

    // 交易相关
    OrderUpdate(Order),
    TradeUpdate(shared_models::trading::Trade),
    PositionUpdate(Position),
    BalanceUpdate(Balance),

    // 策略相关
    StrategySignal(StrategySignal),
    StrategyUpdate(Strategy),
    BacktestUpdate(BacktestResult),

    // 风险管理
    RiskAlert(RiskEvent),
    RiskUpdate(RiskMetric),

    // 系统消息
    Ping,
    Pong,
    Error { message: String },
    Notification { title: String, message: String },

    // 自定义消息
    Custom { event: String, payload: serde_json::Value },
}

/// WebSocket客户端
pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    url: String,
    subscriptions: Vec<String>,
}

impl WsClient {
    /// 连接到WebSocket服务器
    pub async fn connect(url: &str) -> Result<Self> {
        let (stream, _) = connect_async(url).await?;
        
        Ok(Self {
            stream,
            url: url.to_string(),
            subscriptions: Vec::new(),
        })
    }

    /// 发送消息
    pub async fn send(&mut self, message: &WsMessage) -> Result<()> {
        let json = serde_json::to_string(message)?;
        self.stream.send(Message::Text(json)).await?;
        Ok(())
    }

    /// 接收消息
    pub async fn receive(&mut self) -> Result<Option<WsMessage>> {
        while let Some(message) = self.stream.next().await {
            match message? {
                Message::Text(text) => {
                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_message) => return Ok(Some(ws_message)),
                        Err(e) => {
                            tracing::warn!("Failed to parse WebSocket message: {}", e);
                            continue;
                        }
                    }
                }
                Message::Binary(data) => {
                    // 处理二进制消息
                    tracing::debug!("Received binary message: {} bytes", data.len());
                }
                Message::Ping(data) => {
                    self.stream.send(Message::Pong(data)).await?;
                }
                Message::Pong(_) => {
                    // 处理pong消息
                }
                Message::Close(_) => {
                    tracing::info!("WebSocket connection closed");
                    break;
                }
                Message::Frame(_) => {
                    // 处理原始帧
                }
            }
        }
        Ok(None)
    }

    /// 认证
    pub async fn authenticate(&mut self, token: &str) -> Result<()> {
        let auth_message = WsMessage::Auth {
            token: token.to_string(),
        };
        self.send(&auth_message).await?;
        Ok(())
    }

    /// 订阅频道
    pub async fn subscribe(&mut self, channels: Vec<String>) -> Result<()> {
        let subscribe_message = WsMessage::Subscribe {
            channels: channels.clone(),
        };
        self.send(&subscribe_message).await?;
        self.subscriptions.extend(channels);
        Ok(())
    }

    /// 取消订阅频道
    pub async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<()> {
        let unsubscribe_message = WsMessage::Unsubscribe {
            channels: channels.clone(),
        };
        self.send(&unsubscribe_message).await?;
        self.subscriptions.retain(|c| !channels.contains(c));
        Ok(())
    }

    /// 发送ping
    pub async fn ping(&mut self) -> Result<()> {
        self.send(&WsMessage::Ping).await?;
        Ok(())
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        self.stream.close(None).await?;
        Ok(())
    }

    /// 获取当前订阅
    pub fn subscriptions(&self) -> &[String] {
        &self.subscriptions
    }

    /// 重新连接
    pub async fn reconnect(&mut self) -> Result<()> {
        let (stream, _) = connect_async(&self.url).await?;
        self.stream = stream;
        Ok(())
    }
}

/// WebSocket服务器
pub struct WsServer {
    connections: HashMap<Uuid, WsConnection>,
    channels: HashMap<String, Vec<Uuid>>,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    /// 添加连接
    pub fn add_connection(&mut self, connection_id: Uuid, connection: WsConnection) {
        self.connections.insert(connection_id, connection);
    }

    /// 移除连接
    pub fn remove_connection(&mut self, connection_id: &Uuid) {
        if let Some(connection) = self.connections.remove(connection_id) {
            // 从所有频道中移除该连接
            for channel_subscribers in self.channels.values_mut() {
                channel_subscribers.retain(|id| id != connection_id);
            }
        }
    }

    /// 订阅频道
    pub fn subscribe(&mut self, connection_id: Uuid, channels: Vec<String>) {
        for channel in channels {
            self.channels
                .entry(channel)
                .or_insert_with(Vec::new)
                .push(connection_id);
        }
    }

    /// 取消订阅频道
    pub fn unsubscribe(&mut self, connection_id: Uuid, channels: Vec<String>) {
        for channel in channels {
            if let Some(subscribers) = self.channels.get_mut(&channel) {
                subscribers.retain(|id| *id != connection_id);
            }
        }
    }

    /// 广播消息到频道
    pub async fn broadcast_to_channel(&mut self, channel: &str, message: &WsMessage) -> Result<()> {
        if let Some(subscribers) = self.channels.get(channel) {
            let message_json = serde_json::to_string(message)?;
            
            for connection_id in subscribers.clone() {
                if let Some(connection) = self.connections.get_mut(&connection_id) {
                    if let Err(e) = connection.send(&message_json).await {
                        tracing::error!("Failed to send message to connection {}: {}", connection_id, e);
                        // 标记连接为需要移除
                    }
                }
            }
        }
        Ok(())
    }

    /// 发送消息到特定连接
    pub async fn send_to_connection(&mut self, connection_id: &Uuid, message: &WsMessage) -> Result<()> {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            let message_json = serde_json::to_string(message)?;
            connection.send(&message_json).await?;
        }
        Ok(())
    }

    /// 获取连接数
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// 获取频道订阅者数
    pub fn channel_subscriber_count(&self, channel: &str) -> usize {
        self.channels.get(channel).map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket连接
pub struct WsConnection {
    id: Uuid,
    user_id: Option<String>,
    authenticated: bool,
    subscriptions: Vec<String>,
    last_ping: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl WsConnection {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            user_id: None,
            authenticated: false,
            subscriptions: Vec::new(),
            last_ping: Utc::now(),
            created_at: Utc::now(),
        }
    }

    /// 认证连接
    pub fn authenticate(&mut self, user_id: String) {
        self.user_id = Some(user_id);
        self.authenticated = true;
    }

    /// 添加订阅
    pub fn add_subscription(&mut self, channel: String) {
        if !self.subscriptions.contains(&channel) {
            self.subscriptions.push(channel);
        }
    }

    /// 移除订阅
    pub fn remove_subscription(&mut self, channel: &str) {
        self.subscriptions.retain(|c| c != channel);
    }

    /// 更新ping时间
    pub fn update_ping(&mut self) {
        self.last_ping = Utc::now();
    }

    /// 检查是否超时
    pub fn is_timeout(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        (now - self.last_ping).num_seconds() > timeout_seconds
    }

    /// 发送消息（需要实际的WebSocket流实现）
    pub async fn send(&mut self, message: &str) -> Result<()> {
        // 这里需要实际的WebSocket流实现
        // 为了编译通过，暂时返回Ok
        tracing::debug!("Sending message to connection {}: {}", self.id, message);
        Ok(())
    }

    // Getters
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Option<&String> {
        self.user_id.as_ref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn subscriptions(&self) -> &[String] {
        &self.subscriptions
    }

    pub fn last_ping(&self) -> DateTime<Utc> {
        self.last_ping
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// WebSocket频道管理器
pub struct ChannelManager {
    channels: HashMap<String, ChannelInfo>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// 创建频道
    pub fn create_channel(&mut self, name: String, description: Option<String>) {
        let channel_info = ChannelInfo {
            name: name.clone(),
            description,
            subscribers: Vec::new(),
            created_at: Utc::now(),
            message_count: 0,
        };
        self.channels.insert(name, channel_info);
    }

    /// 删除频道
    pub fn delete_channel(&mut self, name: &str) {
        self.channels.remove(name);
    }

    /// 获取频道信息
    pub fn get_channel(&self, name: &str) -> Option<&ChannelInfo> {
        self.channels.get(name)
    }

    /// 获取所有频道
    pub fn get_all_channels(&self) -> Vec<&ChannelInfo> {
        self.channels.values().collect()
    }

    /// 添加订阅者
    pub fn add_subscriber(&mut self, channel: &str, connection_id: Uuid) {
        if let Some(channel_info) = self.channels.get_mut(channel) {
            if !channel_info.subscribers.contains(&connection_id) {
                channel_info.subscribers.push(connection_id);
            }
        }
    }

    /// 移除订阅者
    pub fn remove_subscriber(&mut self, channel: &str, connection_id: &Uuid) {
        if let Some(channel_info) = self.channels.get_mut(channel) {
            channel_info.subscribers.retain(|id| id != connection_id);
        }
    }

    /// 增加消息计数
    pub fn increment_message_count(&mut self, channel: &str) {
        if let Some(channel_info) = self.channels.get_mut(channel) {
            channel_info.message_count += 1;
        }
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 频道信息
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub name: String,
    pub description: Option<String>,
    pub subscribers: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub message_count: u64,
}

/// WebSocket消息路由器
pub struct MessageRouter {
    handlers: HashMap<String, Box<dyn MessageHandler>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册消息处理器
    pub fn register_handler(&mut self, message_type: String, handler: Box<dyn MessageHandler>) {
        self.handlers.insert(message_type, handler);
    }

    /// 路由消息
    pub async fn route_message(&mut self, message: &WsMessage, connection_id: Uuid) -> Result<Option<WsMessage>> {
        let message_type = match message {
            WsMessage::Auth { .. } => "auth",
            WsMessage::Subscribe { .. } => "subscribe",
            WsMessage::Unsubscribe { .. } => "unsubscribe",
            WsMessage::Ping => "ping",
            WsMessage::Custom { event, .. } => event,
            _ => "unknown",
        };

        if let Some(handler) = self.handlers.get_mut(message_type) {
            handler.handle_message(message, connection_id).await
        } else {
            Ok(Some(WsMessage::Error {
                message: format!("Unknown message type: {}", message_type),
            }))
        }
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 消息处理器trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&mut self, message: &WsMessage, connection_id: Uuid) -> Result<Option<WsMessage>>;
}

/// 认证处理器
pub struct AuthHandler {
    // JWT验证器等
}

#[async_trait::async_trait]
impl MessageHandler for AuthHandler {
    async fn handle_message(&mut self, message: &WsMessage, _connection_id: Uuid) -> Result<Option<WsMessage>> {
        match message {
            WsMessage::Auth { token } => {
                // 验证token
                if self.validate_token(token).await? {
                    Ok(Some(WsMessage::AuthSuccess {
                        user_id: "user123".to_string(), // 从token中提取
                    }))
                } else {
                    Ok(Some(WsMessage::AuthError {
                        message: "Invalid token".to_string(),
                    }))
                }
            }
            _ => Ok(None),
        }
    }
}

impl AuthHandler {
    pub fn new() -> Self {
        Self {}
    }

    async fn validate_token(&self, _token: &str) -> Result<bool> {
        // 实现token验证逻辑
        Ok(true)
    }
}

impl Default for AuthHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let message = WsMessage::Ping;
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            WsMessage::Ping => assert!(true),
            _ => assert!(false, "Message type mismatch"),
        }
    }

    #[test]
    fn test_ws_connection() {
        let connection_id = Uuid::new_v4();
        let mut connection = WsConnection::new(connection_id);
        
        assert_eq!(connection.id(), connection_id);
        assert!(!connection.is_authenticated());
        
        connection.authenticate("user123".to_string());
        assert!(connection.is_authenticated());
        assert_eq!(connection.user_id(), Some(&"user123".to_string()));
    }

    #[test]
    fn test_channel_manager() {
        let mut manager = ChannelManager::new();
        
        manager.create_channel("test_channel".to_string(), Some("Test channel".to_string()));
        
        let channel = manager.get_channel("test_channel").unwrap();
        assert_eq!(channel.name, "test_channel");
        assert_eq!(channel.description, Some("Test channel".to_string()));
        
        let connection_id = Uuid::new_v4();
        manager.add_subscriber("test_channel", connection_id);
        
        let channel = manager.get_channel("test_channel").unwrap();
        assert_eq!(channel.subscribers.len(), 1);
        assert!(channel.subscribers.contains(&connection_id));
    }
}