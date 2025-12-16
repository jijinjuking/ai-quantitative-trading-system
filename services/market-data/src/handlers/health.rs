use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error};

use crate::AppState;
use super::{ApiResponse, ApiError};

/// 基础健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: i64,
    pub version: String,
    pub uptime_seconds: u64,
}

/// 详细健康检查响应
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub timestamp: i64,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
    pub metrics: HealthMetrics,
}

/// 组件健康状态
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
    pub last_check: i64,
    pub details: Option<serde_json::Value>,
}

/// 健康指标
#[derive(Debug, Serialize)]
pub struct HealthMetrics {
    pub total_events_processed: u64,
    pub events_per_second: f64,
    pub error_rate: f64,
    pub active_connections: u64,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
}

/// 基础健康检查处理器
pub async fn health_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<HealthResponse>>, ApiError> {
    debug!("Health check requested");

    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 详细健康检查处理器
pub async fn detailed_health_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<DetailedHealthResponse>>, ApiError> {
    debug!("Detailed health check requested");

    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut components = HashMap::new();
    let mut overall_status = "healthy";

    // 检查存储管理器健康状态
    match state.storage_manager.health_check().await {
        Ok(storage_health) => {
            if !storage_health.is_healthy {
                overall_status = "degraded";
            }
            
            components.insert("storage_manager".to_string(), ComponentHealth {
                status: if storage_health.is_healthy { "healthy" } else { "unhealthy" }.to_string(),
                message: None,
                last_check: chrono::Utc::now().timestamp_millis(),
                details: Some(serde_json::to_value(&storage_health).unwrap_or_default()),
            });

            // 检查各个存储后端
            for (backend_name, backend_health) in storage_health.backend_healths {
                components.insert(format!("storage_{}", backend_name), ComponentHealth {
                    status: if backend_health.is_healthy { "healthy" } else { "unhealthy" }.to_string(),
                    message: Some(backend_health.connection_status.clone()),
                    last_check: chrono::Utc::now().timestamp_millis(),
                    details: Some(serde_json::to_value(&backend_health).unwrap_or_default()),
                });
            }
        }
        Err(e) => {
            overall_status = "unhealthy";
            error!("Storage manager health check failed: {}", e);
            
            components.insert("storage_manager".to_string(), ComponentHealth {
                status: "unhealthy".to_string(),
                message: Some(format!("Health check failed: {}", e)),
                last_check: chrono::Utc::now().timestamp_millis(),
                details: None,
            });
        }
    }

    // 检查数据处理器健康状态
    let processor_health = state.data_processor.health_check().await;
    if !processor_health.is_healthy {
        overall_status = "degraded";
    }

    components.insert("data_processor".to_string(), ComponentHealth {
        status: if processor_health.is_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: Some(format!("Error rate: {:.2}%", processor_health.error_rate)),
        last_check: chrono::Utc::now().timestamp_millis(),
        details: Some(serde_json::to_value(&processor_health).unwrap_or_default()),
    });

    // 检查交易所连接管理器健康状态
    let exchange_health = state.exchange_manager.health_check().await;
    let healthy_exchanges = exchange_health.iter().filter(|(_, h)| h.is_healthy).count();
    let total_exchanges = exchange_health.len();

    if healthy_exchanges == 0 && total_exchanges > 0 {
        overall_status = "unhealthy";
    } else if healthy_exchanges < total_exchanges {
        overall_status = "degraded";
    }

    components.insert("exchange_manager".to_string(), ComponentHealth {
        status: if healthy_exchanges == total_exchanges { "healthy" } else { "degraded" }.to_string(),
        message: Some(format!("{}/{} exchanges healthy", healthy_exchanges, total_exchanges)),
        last_check: chrono::Utc::now().timestamp_millis(),
        details: Some(serde_json::to_value(&exchange_health).unwrap_or_default()),
    });

    // 收集指标
    let processor_stats = state.data_processor.get_stats();
    let storage_stats = state.storage_manager.get_stats().await;

    let metrics = HealthMetrics {
        total_events_processed: processor_stats.total_events,
        events_per_second: processor_stats.events_per_second,
        error_rate: processor_stats.error_rate(),
        active_connections: 0, // TODO: 从WebSocket服务器获取
        memory_usage_mb: get_memory_usage(),
        cpu_usage_percent: get_cpu_usage(),
    };

    let response = DetailedHealthResponse {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        components,
        metrics,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 获取内存使用情况（简化实现）
fn get_memory_usage() -> Option<f64> {
    // 在实际实现中，可以使用系统调用或第三方库获取内存使用情况
    // 这里返回None表示不可用
    None
}

/// 获取CPU使用情况（简化实现）
fn get_cpu_usage() -> Option<f64> {
    // 在实际实现中，可以使用系统调用或第三方库获取CPU使用情况
    // 这里返回None表示不可用
    None
}

/// 健康检查配置
#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            timeout_seconds: 10,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

/// 健康检查器
pub struct HealthChecker {
    config: HealthCheckConfig,
    failure_count: std::sync::atomic::AtomicU32,
    success_count: std::sync::atomic::AtomicU32,
    last_check: std::sync::RwLock<Option<std::time::Instant>>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            failure_count: std::sync::atomic::AtomicU32::new(0),
            success_count: std::sync::atomic::AtomicU32::new(0),
            last_check: std::sync::RwLock::new(None),
        }
    }

    /// 执行健康检查
    pub async fn check_health(&self, state: &AppState) -> bool {
        if !self.config.enabled {
            return true;
        }

        let start_time = std::time::Instant::now();
        *self.last_check.write().unwrap() = Some(start_time);

        // 执行各种健康检查
        let mut checks_passed = 0;
        let mut total_checks = 0;

        // 检查存储管理器
        total_checks += 1;
        if let Ok(storage_health) = state.storage_manager.health_check().await {
            if storage_health.is_healthy {
                checks_passed += 1;
            }
        }

        // 检查数据处理器
        total_checks += 1;
        let processor_health = state.data_processor.health_check().await;
        if processor_health.is_healthy {
            checks_passed += 1;
        }

        // 检查交易所连接
        total_checks += 1;
        let exchange_health = state.exchange_manager.health_check().await;
        let healthy_exchanges = exchange_health.iter().filter(|(_, h)| h.is_healthy).count();
        if healthy_exchanges > 0 {
            checks_passed += 1;
        }

        let is_healthy = checks_passed >= total_checks / 2; // 至少一半的检查通过

        // 更新计数器
        if is_healthy {
            self.success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.success_count.store(0, std::sync::atomic::Ordering::Relaxed);
        }

        let elapsed = start_time.elapsed();
        debug!("Health check completed in {:?}: {} healthy", elapsed, is_healthy);

        is_healthy
    }

    /// 获取当前健康状态
    pub fn is_healthy(&self) -> bool {
        let failure_count = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
        failure_count < self.config.failure_threshold
    }

    /// 获取最后检查时间
    pub fn last_check_time(&self) -> Option<std::time::Instant> {
        *self.last_check.read().unwrap()
    }

    /// 启动定期健康检查任务
    pub fn start_periodic_check(&self, state: AppState) {
        if !self.config.enabled {
            return;
        }

        let config = self.config.clone();
        let checker = Self::new(config.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(config.interval_seconds)
            );

            loop {
                interval.tick().await;
                
                let timeout = tokio::time::timeout(
                    std::time::Duration::from_secs(config.timeout_seconds),
                    checker.check_health(&state)
                );

                match timeout.await {
                    Ok(is_healthy) => {
                        if !is_healthy {
                            error!("Health check failed");
                        }
                    }
                    Err(_) => {
                        error!("Health check timed out");
                        checker.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        });

        debug!("Periodic health check started with interval {}s", config.interval_seconds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_config() {
        let config = HealthCheckConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_seconds, 30);
        assert_eq!(config.timeout_seconds, 10);
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.success_threshold, 2);
    }

    #[test]
    fn test_health_checker() {
        let config = HealthCheckConfig::default();
        let checker = HealthChecker::new(config);
        
        // 初始状态应该是健康的
        assert!(checker.is_healthy());
        
        // 模拟失败
        for _ in 0..2 {
            checker.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        assert!(checker.is_healthy()); // 还没达到阈值
        
        checker.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(!checker.is_healthy()); // 达到失败阈值
    }

    #[test]
    fn test_component_health() {
        let health = ComponentHealth {
            status: "healthy".to_string(),
            message: Some("All systems operational".to_string()),
            last_check: chrono::Utc::now().timestamp_millis(),
            details: None,
        };

        assert_eq!(health.status, "healthy");
        assert!(health.message.is_some());
    }
}