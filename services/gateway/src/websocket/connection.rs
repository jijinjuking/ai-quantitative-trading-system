use anyhow::Result;
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket连接状态
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

/// WebSocket连接
#[derive(Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub service_name: String,
    pub target_url: String,
    pub state: Arc<RwLock<ConnectionState>>,
    pub created_at: Instant,
    pub last_activity: Arc<RwLock<Instant>>,
    pub message_count: Arc<RwLock<u64>>,
    pub error_count: Arc<RwLock<u64>>,
}

impl WebSocketConnection {
    /// 创建新的WebSocket连接
    pub fn new(service_name: String, target_url: String) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Instant::now();

        Self {
            id,
            service_name,
            target_url,
            state: Arc::new(RwLock::new(ConnectionState::Connecting)),
            created_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            message_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
        }
    }

    /// 建立代理连接
    pub async fn establish_proxy(
        &self,
        client_ws: WebSocket,
    ) -> Result<()> {
        info!("Establishing WebSocket proxy for connection: {}", self.id);

        // 连接到目标服务
        let (target_ws, _) = match connect_async(&self.target_url).await {
            Ok(connection) => connection,
            Err(e) => {
                error!("Failed to connect to target WebSocket: {}", e);
                self.set_state(ConnectionState::Error).await;
                return Err(anyhow::anyhow!("Connection failed: {}", e));
            }
        };

        self.set_state(ConnectionState::Connected).await;
        info!("WebSocket proxy established for: {}", self.service_name);

        // 分离读写流
        let (mut client_sender, mut client_receiver) = client_ws.split();
        let (mut target_sender, mut target_receiver) = target_ws.split();

        // 创建消息通道
        let (client_tx, mut client_rx) = mpsc::unbounded_channel::<String>();
        let (target_tx, mut target_rx) = mpsc::unbounded_channel::<String>();

        let connection_id = self.id.clone();
        let message_count = self.message_count.clone();
        let error_count = self.error_count.clone();
        let last_activity = self.last_activity.clone();

        // 客户端到目标服务的消息转发
        let client_to_target = {
            let connection_id = connection_id.clone();
            let message_count = message_count.clone();
            let error_count = error_count.clone();
            let last_activity = last_activity.clone();
            
            tokio::spawn(async move {
                while let Some(msg) = client_receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            debug!("Client->Target text message: {} chars", text.len());
                            if let Err(e) = target_sender.send(TungsteniteMessage::Text(text)).await {
                                error!("Failed to forward text message to target: {}", e);
                                let mut count = error_count.write().await;
                                *count += 1;
                                break;
                            }
                        }
                        Ok(Message::Binary(data)) => {
                            debug!("Client->Target binary message: {} bytes", data.len());
                            if let Err(e) = target_sender.send(TungsteniteMessage::Binary(data)).await {
                                error!("Failed to forward binary message to target: {}", e);
                                let mut count = error_count.write().await;
                                *count += 1;
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("Client closed WebSocket connection: {}", connection_id);
                            let _ = target_sender.send(TungsteniteMessage::Close(None)).await;
                            break;
                        }
                        Ok(Message::Ping(data)) => {
                            debug!("Client ping received");
                            if let Err(e) = target_sender.send(TungsteniteMessage::Ping(data)).await {
                                error!("Failed to forward ping to target: {}", e);
                            }
                        }
                        Ok(Message::Pong(data)) => {
                            debug!("Client pong received");
                            if let Err(e) = target_sender.send(TungsteniteMessage::Pong(data)).await {
                                error!("Failed to forward pong to target: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Client WebSocket error: {}", e);
                            let mut count = error_count.write().await;
                            *count += 1;
                            break;
                        }
                        _ => {
                            // 处理其他消息类型
                            debug!("Unhandled message type");
                        }
                    }

                    // 更新活动时间和消息计数
                    *last_activity.write().await = Instant::now();
                    let mut count = message_count.write().await;
                    *count += 1;
                }
            })
        };

        // 目标服务到客户端的消息转发
        let target_to_client = {
            let connection_id = connection_id.clone();
            let message_count = message_count.clone();
            let error_count = error_count.clone();
            let last_activity = last_activity.clone();
            
            tokio::spawn(async move {
                while let Some(msg) = target_receiver.next().await {
                    match msg {
                        Ok(TungsteniteMessage::Text(text)) => {
                            debug!("Target->Client text message: {} chars", text.len());
                            if let Err(e) = client_sender.send(Message::Text(text)).await {
                                error!("Failed to forward text message to client: {}", e);
                                let mut count = error_count.write().await;
                                *count += 1;
                                break;
                            }
                        }
                        Ok(TungsteniteMessage::Binary(data)) => {
                            debug!("Target->Client binary message: {} bytes", data.len());
                            if let Err(e) = client_sender.send(Message::Binary(data)).await {
                                error!("Failed to forward binary message to client: {}", e);
                                let mut count = error_count.write().await;
                                *count += 1;
                                break;
                            }
                        }
                        Ok(TungsteniteMessage::Close(_)) => {
                            info!("Target closed WebSocket connection: {}", connection_id);
                            let _ = client_sender.send(Message::Close(None)).await;
                            break;
                        }
                        Ok(TungsteniteMessage::Ping(data)) => {
                            debug!("Target ping received");
                            if let Err(e) = client_sender.send(Message::Ping(data)).await {
                                error!("Failed to forward ping to client: {}", e);
                            }
                        }
                        Ok(TungsteniteMessage::Pong(data)) => {
                            debug!("Target pong received");
                            if let Err(e) = client_sender.send(Message::Pong(data)).await {
                                error!("Failed to forward pong to client: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Target WebSocket error: {}", e);
                            let mut count = error_count.write().await;
                            *count += 1;
                            break;
                        }
                        _ => {
                            // 处理其他消息类型
                            debug!("Unhandled target message type");
                        }
                    }

                    // 更新活动时间和消息计数
                    *last_activity.write().await = Instant::now();
                    let mut count = message_count.write().await;
                    *count += 1;
                }
            })
        };

        // 等待任一方向的连接关闭
        tokio::select! {
            _ = client_to_target => {
                debug!("Client to target forwarding ended");
            }
            _ = target_to_client => {
                debug!("Target to client forwarding ended");
            }
        }

        self.set_state(ConnectionState::Disconnected).await;
        info!("WebSocket proxy connection closed: {}", self.id);

        Ok(())
    }

    /// 设置连接状态
    pub async fn set_state(&self, state: ConnectionState) {
        let mut current_state = self.state.write().await;
        *current_state = state;
    }

    /// 获取连接状态
    pub async fn get_state(&self) -> ConnectionState {
        let state = self.state.read().await;
        state.clone()
    }

    /// 获取连接统计信息
    pub async fn get_stats(&self) -> ConnectionStats {
        let state = self.get_state().await;
        let message_count = *self.message_count.read().await;
        let error_count = *self.error_count.read().await;
        let last_activity = *self.last_activity.read().await;
        let uptime = self.created_at.elapsed();

        ConnectionStats {
            id: self.id.clone(),
            service_name: self.service_name.clone(),
            target_url: self.target_url.clone(),
            state,
            message_count,
            error_count,
            uptime,
            last_activity,
        }
    }

    /// 检查连接是否活跃
    pub async fn is_active(&self, timeout: Duration) -> bool {
        let last_activity = *self.last_activity.read().await;
        let state = self.get_state().await;
        
        state == ConnectionState::Connected && 
        last_activity.elapsed() < timeout
    }

    /// 发送心跳
    pub async fn send_heartbeat(&self) -> Result<()> {
        // 心跳逻辑可以根据需要实现
        *self.last_activity.write().await = Instant::now();
        Ok(())
    }
}

/// 连接统计信息
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStats {
    pub id: String,
    pub service_name: String,
    pub target_url: String,
    pub state: ConnectionState,
    pub message_count: u64,
    pub error_count: u64,
    #[serde(serialize_with = "serialize_duration")]
    pub uptime: Duration,
    #[serde(skip)]
    pub last_activity: Instant,
}

/// 序列化Duration为秒数
fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

/// 连接配置
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionConfig {
    pub connect_timeout: Duration,
    pub message_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub max_message_size: usize,
    pub max_connections_per_service: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            message_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(30),
            max_message_size: 1024 * 1024, // 1MB
            max_connections_per_service: 1000,
        }
    }
}