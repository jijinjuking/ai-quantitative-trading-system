use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 通用响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

/// 分页请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: None,
            sort_order: Some(SortOrder::Desc),
        }
    }
}

/// 分页响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 排序顺序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 交易所枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Exchange {
    Binance,
    OKX,
    Huobi,
    Bybit,
    KuCoin,
    Gate,
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::Binance => write!(f, "binance"),
            Exchange::OKX => write!(f, "okx"),
            Exchange::Huobi => write!(f, "huobi"),
            Exchange::Bybit => write!(f, "bybit"),
            Exchange::KuCoin => write!(f, "kucoin"),
            Exchange::Gate => write!(f, "gate"),
        }
    }
}

impl Exchange {
    /// 检查是否为空（枚举类型永远不为空）
    pub fn is_empty(&self) -> bool {
        false
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Exchange::Binance => "binance",
            Exchange::OKX => "okx",
            Exchange::Huobi => "huobi",
            Exchange::Bybit => "bybit",
            Exchange::KuCoin => "kucoin",
            Exchange::Gate => "gate",
        }
    }
}

/// 交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub exchange: Exchange,
    pub status: SymbolStatus,
    pub min_qty: Decimal,
    pub max_qty: Decimal,
    pub step_size: Decimal,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub tick_size: Decimal,
    pub min_notional: Decimal,
}

/// 交易对状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolStatus {
    Trading,
    PreTrading,
    PostTrading,
    EndOfDay,
    Halt,
    AuctionMatch,
    Break,
}

/// 时间间隔枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Interval {
    #[serde(rename = "1s")]
    OneSecond,
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "3m")]
    ThreeMinutes,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    TwoHours,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "6h")]
    SixHours,
    #[serde(rename = "8h")]
    EightHours,
    #[serde(rename = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "3d")]
    ThreeDays,
    #[serde(rename = "1w")]
    OneWeek,
    #[serde(rename = "1M")]
    OneMonth,
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Interval::OneSecond => "1s",
            Interval::OneMinute => "1m",
            Interval::ThreeMinutes => "3m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::OneHour => "1h",
            Interval::TwoHours => "2h",
            Interval::FourHours => "4h",
            Interval::SixHours => "6h",
            Interval::EightHours => "8h",
            Interval::TwelveHours => "12h",
            Interval::OneDay => "1d",
            Interval::ThreeDays => "3d",
            Interval::OneWeek => "1w",
            Interval::OneMonth => "1M",
        };
        write!(f, "{}", s)
    }
}

impl Interval {
    /// 检查是否为空（枚举类型永远不为空）
    pub fn is_empty(&self) -> bool {
        false
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::OneSecond => "1s",
            Interval::OneMinute => "1m",
            Interval::ThreeMinutes => "3m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::OneHour => "1h",
            Interval::TwoHours => "2h",
            Interval::FourHours => "4h",
            Interval::SixHours => "6h",
            Interval::EightHours => "8h",
            Interval::TwelveHours => "12h",
            Interval::OneDay => "1d",
            Interval::ThreeDays => "3d",
            Interval::OneWeek => "1w",
            Interval::OneMonth => "1M",
        }
    }

    pub fn to_seconds(&self) -> u64 {
        match self {
            Interval::OneSecond => 1,
            Interval::OneMinute => 60,
            Interval::ThreeMinutes => 180,
            Interval::FiveMinutes => 300,
            Interval::FifteenMinutes => 900,
            Interval::ThirtyMinutes => 1800,
            Interval::OneHour => 3600,
            Interval::TwoHours => 7200,
            Interval::FourHours => 14400,
            Interval::SixHours => 21600,
            Interval::EightHours => 28800,
            Interval::TwelveHours => 43200,
            Interval::OneDay => 86400,
            Interval::ThreeDays => 259200,
            Interval::OneWeek => 604800,
            Interval::OneMonth => 2592000, // 30 days
        }
    }
}

/// 配置项结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
    pub is_sensitive: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub service_name: String,
    pub status: ServiceStatus,
    pub version: String,
    pub uptime: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 服务状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// 数据质量枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQuality {
    /// 正常数据 - 连续性良好
    #[serde(rename = "normal")]
    Normal,
    /// 可疑数据 - gap后首次到达的数据
    #[serde(rename = "suspect")]
    Suspect,
    /// 恢复数据 - 通过回补等方式恢复的数据
    #[serde(rename = "recovered")]
    Recovered,
}

impl DataQuality {
    /// 转换为字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            DataQuality::Normal => "normal",
            DataQuality::Suspect => "suspect",
            DataQuality::Recovered => "recovered",
        }
    }
}

impl Default for DataQuality {
    fn default() -> Self {
        DataQuality::Normal
    }
}

impl std::fmt::Display for DataQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 错误类型定义
#[derive(thiserror::Error, Debug)]
pub enum CommonError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
