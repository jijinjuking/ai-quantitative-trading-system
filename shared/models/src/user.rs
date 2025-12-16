use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::common::Exchange;

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub two_factor_enabled: bool,
    pub roles: Vec<Role>,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
}

/// 用户状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Banned,
    PendingVerification,
}

/// 用户角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// 权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

/// 用户偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub language: String,
    pub timezone: String,
    pub currency: String,
    pub theme: String,
    pub notifications: NotificationSettings,
    pub trading: TradingSettings,
    pub display: DisplaySettings,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            currency: "USD".to_string(),
            theme: "light".to_string(),
            notifications: NotificationSettings::default(),
            trading: TradingSettings::default(),
            display: DisplaySettings::default(),
        }
    }
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub trade_notifications: bool,
    pub price_alerts: bool,
    pub system_notifications: bool,
    pub marketing_emails: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_enabled: true,
            sms_enabled: false,
            push_enabled: true,
            trade_notifications: true,
            price_alerts: true,
            system_notifications: true,
            marketing_emails: false,
        }
    }
}

/// 交易设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSettings {
    pub default_order_type: String,
    pub default_time_in_force: String,
    pub confirm_orders: bool,
    pub auto_cancel_orders: bool,
    pub risk_warnings: bool,
    pub advanced_orders: bool,
}

impl Default for TradingSettings {
    fn default() -> Self {
        Self {
            default_order_type: "LIMIT".to_string(),
            default_time_in_force: "GTC".to_string(),
            confirm_orders: true,
            auto_cancel_orders: false,
            risk_warnings: true,
            advanced_orders: false,
        }
    }
}

/// 显示设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub decimal_places: u8,
    pub show_zero_balances: bool,
    pub chart_type: String,
    pub chart_interval: String,
    pub show_order_book: bool,
    pub show_trade_history: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            decimal_places: 8,
            show_zero_balances: false,
            chart_type: "candlestick".to_string(),
            chart_interval: "1h".to_string(),
            show_order_book: true,
            show_trade_history: true,
        }
    }
}

/// API密钥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: Exchange,
    pub name: String,
    pub key: String,
    pub secret: String,             // 加密存储
    pub passphrase: Option<String>, // 加密存储
    pub permissions: Vec<ApiPermission>,
    pub is_active: bool,
    pub is_testnet: bool,
    pub ip_whitelist: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

/// API权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiPermission {
    Read,
    Trade,
    Withdraw,
    Futures,
    Margin,
}

/// 用户会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub refresh_token: String,
    pub device_info: DeviceInfo,
    pub ip_address: String,
    pub user_agent: String,
    pub is_active: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub os: String,
    pub browser: String,
    pub location: Option<String>,
}

/// 用户活动日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub description: String,
    pub ip_address: String,
    pub user_agent: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 活动类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Login,
    Logout,
    PasswordChange,
    EmailChange,
    PhoneChange,
    TwoFactorEnabled,
    TwoFactorDisabled,
    ApiKeyCreated,
    ApiKeyDeleted,
    OrderPlaced,
    OrderCancelled,
    TradeExecuted,
    Withdrawal,
    Deposit,
    SecurityAlert,
}

/// 用户注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub referral_code: Option<String>,
    pub terms_accepted: bool,
    pub privacy_accepted: bool,
}

/// 用户登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
    pub two_factor_code: Option<String>,
    pub remember_me: bool,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

/// 密码重置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// 密码重置确认
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

/// 用户更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<UserPreferences>,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: usize,  // 过期时间
    pub iat: usize,  // 签发时间
    pub nbf: usize,  // 生效时间
    pub iss: String, // 签发者
    pub aud: String, // 受众
}

/// 双因子认证设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorAuth {
    pub id: Uuid,
    pub user_id: Uuid,
    pub method: TwoFactorMethod,
    pub secret: String,            // 加密存储
    pub backup_codes: Vec<String>, // 加密存储
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

/// 双因子认证方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TwoFactorMethod {
    TOTP,  // Time-based One-Time Password
    SMS,   // SMS验证码
    Email, // 邮件验证码
}
