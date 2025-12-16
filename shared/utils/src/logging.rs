use anyhow::Result;
use serde_json::json;
use std::io;
use tracing::{Event, Subscriber};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    fmt::{self, format::Writer, time::FormatTime, FmtContext, FormatEvent, FormatFields},
    EnvFilter,
};

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_config: Option<FileConfig>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Stdout,
            file_config: None,
        }
    }
}

/// 日志格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    Pretty,
    Compact,
    Json,
    Custom,
}

/// 日志输出
#[derive(Debug, Clone)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File,
    Both,
}

/// 文件配置
#[derive(Debug, Clone)]
pub struct FileConfig {
    pub directory: String,
    pub filename_prefix: String,
    pub rotation: FileRotation,
    pub max_files: Option<usize>,
}

/// 文件轮转
#[derive(Debug, Clone)]
pub enum FileRotation {
    Never,
    Minutely,
    Hourly,
    Daily,
}

/// 日志初始化器
pub struct LoggingInitializer;

impl LoggingInitializer {
    /// 初始化日志系统
    pub fn init(config: LoggingConfig) -> Result<()> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

        match config.output {
            LogOutput::Stdout => match config.format {
                LogFormat::Pretty => {
                    tracing_subscriber::fmt()
                        .pretty()
                        .with_env_filter(env_filter)
                        .init();
                }
                LogFormat::Compact => {
                    tracing_subscriber::fmt()
                        .compact()
                        .with_env_filter(env_filter)
                        .init();
                }
                LogFormat::Json => {
                    tracing_subscriber::fmt()
                        .json()
                        .with_env_filter(env_filter)
                        .init();
                }
                LogFormat::Custom => {
                    tracing_subscriber::fmt()
                        .event_format(CustomFormatter::new())
                        .with_env_filter(env_filter)
                        .init();
                }
            },
            LogOutput::Stderr => match config.format {
                LogFormat::Pretty => {
                    tracing_subscriber::fmt()
                        .pretty()
                        .with_writer(std::io::stderr)
                        .with_env_filter(env_filter)
                        .init();
                }
                LogFormat::Compact => {
                    tracing_subscriber::fmt()
                        .compact()
                        .with_writer(std::io::stderr)
                        .with_env_filter(env_filter)
                        .init();
                }
                LogFormat::Json => {
                    tracing_subscriber::fmt()
                        .json()
                        .with_writer(std::io::stderr)
                        .with_env_filter(env_filter)
                        .init();
                }
                LogFormat::Custom => {
                    tracing_subscriber::fmt()
                        .event_format(CustomFormatter::new())
                        .with_writer(std::io::stderr)
                        .with_env_filter(env_filter)
                        .init();
                }
            },
            LogOutput::File => {
                if let Some(file_config) = config.file_config {
                    let file_appender = Self::create_file_appender(&file_config);
                    let (non_blocking, _guard) = non_blocking(file_appender);
                    std::mem::forget(_guard); // 防止guard被丢弃

                    match config.format {
                        LogFormat::Pretty => {
                            tracing_subscriber::fmt()
                                .pretty()
                                .with_writer(non_blocking)
                                .with_env_filter(env_filter)
                                .init();
                        }
                        LogFormat::Compact => {
                            tracing_subscriber::fmt()
                                .compact()
                                .with_writer(non_blocking)
                                .with_env_filter(env_filter)
                                .init();
                        }
                        LogFormat::Json => {
                            tracing_subscriber::fmt()
                                .json()
                                .with_writer(non_blocking)
                                .with_env_filter(env_filter)
                                .init();
                        }
                        LogFormat::Custom => {
                            tracing_subscriber::fmt()
                                .event_format(CustomFormatter::new())
                                .with_writer(non_blocking)
                                .with_env_filter(env_filter)
                                .init();
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("File config is required for file output"));
                }
            }
            LogOutput::Both => {
                // 简化实现：使用stdout输出
                tracing_subscriber::fmt()
                    .pretty()
                    .with_env_filter(env_filter)
                    .init();
            }
        }

        Ok(())
    }

    /// 创建文件appender
    fn create_file_appender(file_config: &FileConfig) -> rolling::RollingFileAppender {
        match file_config.rotation {
            FileRotation::Never => rolling::never(
                &file_config.directory,
                format!("{}.log", file_config.filename_prefix),
            ),
            FileRotation::Minutely => {
                rolling::minutely(&file_config.directory, &file_config.filename_prefix)
            }
            FileRotation::Hourly => {
                rolling::hourly(&file_config.directory, &file_config.filename_prefix)
            }
            FileRotation::Daily => {
                rolling::daily(&file_config.directory, &file_config.filename_prefix)
            }
        }
    }

    /// 快速初始化（开发环境）
    pub fn init_dev() -> Result<()> {
        let config = LoggingConfig {
            level: "debug".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Stdout,
            file_config: None,
        };

        Self::init(config)
    }

    /// 快速初始化（生产环境）
    pub fn init_prod(log_dir: &str) -> Result<()> {
        let config = LoggingConfig {
            level: "info".to_string(),
            format: LogFormat::Json,
            output: LogOutput::Both,
            file_config: Some(FileConfig {
                directory: log_dir.to_string(),
                filename_prefix: "app".to_string(),
                rotation: FileRotation::Daily,
                max_files: Some(30),
            }),
        };

        Self::init(config)
    }
}

/// 自定义格式化器
pub struct CustomFormatter {
    timer: fmt::time::SystemTime,
}

impl CustomFormatter {
    pub fn new() -> Self {
        Self {
            timer: fmt::time::SystemTime,
        }
    }
}

impl Default for CustomFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // 时间戳
        self.timer.format_time(&mut writer)?;
        writer.write_char(' ')?;

        // 日志级别
        let level = event.metadata().level();
        write!(writer, "{:>5} ", level)?;

        // 目标模块
        let target = event.metadata().target();
        write!(writer, "[{}] ", target)?;

        // Span信息
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}:", span.name())?;
            }
            writer.write_char(' ')?;
        }

        // 消息内容
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// 业务日志工具
pub struct BusinessLogger;

impl BusinessLogger {
    /// 记录用户操作
    pub fn log_user_action(user_id: &str, action: &str, details: Option<serde_json::Value>) {
        tracing::info!(
            user_id = user_id,
            action = action,
            details = ?details,
            "User action"
        );
    }

    /// 记录交易操作
    pub fn log_trading_action(
        user_id: &str,
        symbol: &str,
        action: &str,
        amount: Option<&str>,
        price: Option<&str>,
    ) {
        tracing::info!(
            user_id = user_id,
            symbol = symbol,
            action = action,
            amount = amount,
            price = price,
            "Trading action"
        );
    }

    /// 记录API调用
    pub fn log_api_call(
        method: &str,
        path: &str,
        status_code: u16,
        duration_ms: u64,
        user_id: Option<&str>,
    ) {
        tracing::info!(
            method = method,
            path = path,
            status_code = status_code,
            duration_ms = duration_ms,
            user_id = user_id,
            "API call"
        );
    }

    /// 记录系统事件
    pub fn log_system_event(event_type: &str, message: &str, metadata: Option<serde_json::Value>) {
        tracing::info!(
            event_type = event_type,
            message = message,
            metadata = ?metadata,
            "System event"
        );
    }

    /// 记录安全事件
    pub fn log_security_event(
        event_type: &str,
        user_id: Option<&str>,
        ip_address: Option<&str>,
        details: &str,
    ) {
        tracing::warn!(
            event_type = event_type,
            user_id = user_id,
            ip_address = ip_address,
            details = details,
            "Security event"
        );
    }

    /// 记录性能指标
    pub fn log_performance_metric(
        metric_name: &str,
        value: f64,
        unit: &str,
        tags: Option<std::collections::HashMap<String, String>>,
    ) {
        tracing::info!(
            metric_name = metric_name,
            value = value,
            unit = unit,
            tags = ?tags,
            "Performance metric"
        );
    }
}

/// 日志过滤器
pub struct LogFilter;

impl LogFilter {
    /// 过滤敏感信息
    pub fn sanitize_log_data(data: &mut serde_json::Value) {
        match data {
            serde_json::Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    if Self::is_sensitive_field(key) {
                        *value = json!("***REDACTED***");
                    } else {
                        Self::sanitize_log_data(value);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::sanitize_log_data(item);
                }
            }
            _ => {}
        }
    }

    /// 检查是否为敏感字段
    fn is_sensitive_field(field_name: &str) -> bool {
        let sensitive_fields = [
            "password",
            "secret",
            "token",
            "key",
            "api_key",
            "private_key",
            "credit_card",
            "ssn",
            "social_security",
        ];

        let field_lower = field_name.to_lowercase();
        sensitive_fields
            .iter()
            .any(|&sensitive| field_lower.contains(sensitive))
    }

    /// 创建安全的日志值
    pub fn safe_log_value<T: serde::Serialize>(value: &T) -> serde_json::Value {
        let mut json_value = serde_json::to_value(value).unwrap_or(json!(null));
        Self::sanitize_log_data(&mut json_value);
        json_value
    }
}

/// 结构化日志宏
#[macro_export]
macro_rules! log_info {
    ($($field:tt)*) => {
        tracing::info!($($field)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($field:tt)*) => {
        tracing::warn!($($field)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($field:tt)*) => {
        tracing::error!($($field)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($field:tt)*) => {
        tracing::debug!($($field)*)
    };
}

/// 性能监控宏
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!(
            operation = $name,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
        result
    }};
}

/// 错误日志宏
#[macro_export]
macro_rules! log_error_with_context {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = $context,
            "Error occurred"
        );
    };
    ($error:expr, $context:expr, $($field:tt)*) => {
        tracing::error!(
            error = %$error,
            context = $context,
            $($field)*
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_filter() {
        let mut data = json!({
            "username": "testuser",
            "password": "secret123",
            "api_key": "abc123",
            "balance": 1000.0,
            "nested": {
                "token": "xyz789",
                "amount": 500.0
            }
        });

        LogFilter::sanitize_log_data(&mut data);

        assert_eq!(data["username"], "testuser");
        assert_eq!(data["password"], "***REDACTED***");
        assert_eq!(data["api_key"], "***REDACTED***");
        assert_eq!(data["balance"], 1000.0);
        assert_eq!(data["nested"]["token"], "***REDACTED***");
        assert_eq!(data["nested"]["amount"], 500.0);
    }

    #[test]
    fn test_sensitive_field_detection() {
        assert!(LogFilter::is_sensitive_field("password"));
        assert!(LogFilter::is_sensitive_field("API_KEY"));
        assert!(LogFilter::is_sensitive_field("user_token"));
        assert!(!LogFilter::is_sensitive_field("username"));
        assert!(!LogFilter::is_sensitive_field("balance"));
    }
}
