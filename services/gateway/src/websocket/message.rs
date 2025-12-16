use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// WebSocket消息类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
    Control,
}

/// WebSocket消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: String,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub source: String,
    pub target: String,
    pub metadata: MessageMetadata,
}

impl WebSocketMessage {
    /// 创建文本消息
    pub fn text(content: String, source: String, target: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Text,
            payload: content.into_bytes(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source,
            target,
            metadata: MessageMetadata::default(),
        }
    }

    /// 创建二进制消息
    pub fn binary(data: Vec<u8>, source: String, target: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Binary,
            payload: data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source,
            target,
            metadata: MessageMetadata::default(),
        }
    }

    /// 创建Ping消息
    pub fn ping(data: Vec<u8>, source: String, target: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Ping,
            payload: data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source,
            target,
            metadata: MessageMetadata::default(),
        }
    }

    /// 创建Pong消息
    pub fn pong(data: Vec<u8>, source: String, target: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Pong,
            payload: data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source,
            target,
            metadata: MessageMetadata::default(),
        }
    }

    /// 创建关闭消息
    pub fn close(reason: Option<String>, source: String, target: String) -> Self {
        let payload = reason.unwrap_or_default().into_bytes();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Close,
            payload,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source,
            target,
            metadata: MessageMetadata::default(),
        }
    }

    /// 获取消息内容为字符串
    pub fn as_text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.clone())
    }

    /// 获取消息大小
    pub fn size(&self) -> usize {
        self.payload.len()
    }

    /// 检查消息是否过期
    pub fn is_expired(&self, ttl: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        now - self.timestamp > ttl.as_millis() as u64
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.custom.insert(key, value);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// 设置重试配置
    pub fn with_retry(mut self, max_retries: u32, retry_delay: Duration) -> Self {
        self.metadata.retry_config = Some(RetryConfig {
            max_retries,
            retry_delay,
            current_attempt: 0,
        });
        self
    }

    /// 验证消息
    pub fn validate(&self) -> Result<(), MessageValidationError> {
        // 检查消息大小
        if self.payload.len() > 10 * 1024 * 1024 {
            // 10MB限制
            return Err(MessageValidationError::TooLarge);
        }

        // 检查ID格式
        if self.id.is_empty() {
            return Err(MessageValidationError::InvalidId);
        }

        // 检查时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if self.timestamp > now + 60000 {
            // 不能超过当前时间1分钟
            return Err(MessageValidationError::InvalidTimestamp);
        }

        Ok(())
    }
}

/// 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub priority: MessagePriority,
    pub retry_config: Option<RetryConfig>,
    pub compression: Option<CompressionType>,
    pub encryption: Option<EncryptionType>,
    pub custom: std::collections::HashMap<String, String>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            priority: MessagePriority::Normal,
            retry_config: None,
            compression: None,
            encryption: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// 消息优先级
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub current_attempt: u32,
}

/// 压缩类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Deflate,
    Brotli,
}

/// 加密类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionType {
    None,
    AES256,
    ChaCha20,
}

/// 消息验证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum MessageValidationError {
    #[error("Message too large")]
    TooLarge,
    #[error("Invalid message ID")]
    InvalidId,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid payload")]
    InvalidPayload,
}

/// 消息统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub total_messages: u64,
    pub messages_by_type: std::collections::HashMap<MessageType, u64>,
    pub total_bytes: u64,
    pub average_size: f64,
    pub messages_per_second: f64,
    pub error_count: u64,
    pub retry_count: u64,
}

impl MessageStats {
    pub fn new() -> Self {
        Self {
            total_messages: 0,
            messages_by_type: std::collections::HashMap::new(),
            total_bytes: 0,
            average_size: 0.0,
            messages_per_second: 0.0,
            error_count: 0,
            retry_count: 0,
        }
    }

    /// 记录消息
    pub fn record_message(&mut self, message: &WebSocketMessage) {
        self.total_messages += 1;
        self.total_bytes += message.size() as u64;

        *self
            .messages_by_type
            .entry(message.message_type.clone())
            .or_insert(0) += 1;

        self.average_size = self.total_bytes as f64 / self.total_messages as f64;
    }

    /// 记录错误
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// 记录重试
    pub fn record_retry(&mut self) {
        self.retry_count += 1;
    }
}

impl Default for MessageStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 消息队列
#[derive(Debug)]
pub struct MessageQueue {
    messages: std::collections::VecDeque<WebSocketMessage>,
    max_size: usize,
    stats: MessageStats,
}

impl MessageQueue {
    /// 创建新的消息队列
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: std::collections::VecDeque::new(),
            max_size,
            stats: MessageStats::new(),
        }
    }

    /// 添加消息到队列
    pub fn push(&mut self, message: WebSocketMessage) -> Result<(), MessageQueueError> {
        if self.messages.len() >= self.max_size {
            return Err(MessageQueueError::QueueFull);
        }

        // 验证消息
        message
            .validate()
            .map_err(MessageQueueError::ValidationError)?;

        self.stats.record_message(&message);

        // 根据优先级插入消息
        match message.metadata.priority {
            MessagePriority::Critical => self.messages.push_front(message),
            MessagePriority::High => {
                // 插入到高优先级消息之后，普通优先级消息之前
                let mut insert_pos = 0;
                for (i, msg) in self.messages.iter().enumerate() {
                    if msg.metadata.priority != MessagePriority::Critical {
                        insert_pos = i;
                        break;
                    }
                }
                self.messages.insert(insert_pos, message);
            }
            _ => self.messages.push_back(message),
        }

        Ok(())
    }

    /// 从队列中取出消息
    pub fn pop(&mut self) -> Option<WebSocketMessage> {
        self.messages.pop_front()
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// 清空队列
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// 获取统计信息
    pub fn stats(&self) -> &MessageStats {
        &self.stats
    }
}

/// 消息队列错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum MessageQueueError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Validation error: {0}")]
    ValidationError(MessageValidationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = WebSocketMessage::text(
            "Hello, World!".to_string(),
            "client".to_string(),
            "server".to_string(),
        );

        assert_eq!(msg.message_type, MessageType::Text);
        assert_eq!(msg.as_text().unwrap(), "Hello, World!");
        assert_eq!(msg.source, "client");
        assert_eq!(msg.target, "server");
    }

    #[test]
    fn test_message_validation() {
        let msg = WebSocketMessage::text(
            "Valid message".to_string(),
            "client".to_string(),
            "server".to_string(),
        );

        assert!(msg.validate().is_ok());

        // 测试过大的消息
        let large_payload = vec![0u8; 11 * 1024 * 1024]; // 11MB
        let large_msg =
            WebSocketMessage::binary(large_payload, "client".to_string(), "server".to_string());

        assert!(matches!(
            large_msg.validate(),
            Err(MessageValidationError::TooLarge)
        ));
    }

    #[test]
    fn test_message_queue() {
        let mut queue = MessageQueue::new(10);

        let msg1 = WebSocketMessage::text(
            "Normal priority".to_string(),
            "client".to_string(),
            "server".to_string(),
        );

        let msg2 = WebSocketMessage::text(
            "High priority".to_string(),
            "client".to_string(),
            "server".to_string(),
        )
        .with_priority(MessagePriority::High);

        assert!(queue.push(msg1).is_ok());
        assert!(queue.push(msg2).is_ok());

        // 高优先级消息应该先出队
        let popped = queue.pop().unwrap();
        assert_eq!(popped.metadata.priority, MessagePriority::High);
    }
}
