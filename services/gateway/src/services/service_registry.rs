use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::{GatewayConfig, ServiceEndpoint};

/// 服务注册表
#[derive(Clone)]
pub struct ServiceRegistry {
    config: GatewayConfig,
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    pub client: Client,
}

impl ServiceRegistry {
    /// 创建新的服务注册表
    pub fn new(config: GatewayConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
            client,
        }
    }

    /// 注册服务
    pub async fn register_service(&self, name: String, info: ServiceInfo) {
        let mut services = self.services.write().await;
        services.insert(name.clone(), info);
        info!("Service registered: {}", name);
    }

    /// 注销服务
    pub async fn unregister_service(&self, name: &str) {
        let mut services = self.services.write().await;
        if services.remove(name).is_some() {
            info!("Service unregistered: {}", name);
        }
    }

    /// 获取服务信息
    pub async fn get_service(&self, name: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    /// 获取所有服务
    pub async fn get_all_services(&self) -> HashMap<String, ServiceInfo> {
        let services = self.services.read().await;
        services.clone()
    }

    /// 获取健康的服务实例
    pub async fn get_healthy_service(&self, name: &str) -> Option<ServiceInfo> {
        let service = self.get_service(name).await?;
        
        if service.status == ServiceStatus::Healthy {
            Some(service)
        } else {
            None
        }
    }

    /// 健康检查
    pub async fn health_check(&self, service_name: &str) -> Result<()> {
        let endpoint = self.config.get_service_endpoint(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))?;

        let health_url = format!("{}/health", endpoint.url);
        
        match self.client.get(&health_url)
            .timeout(Duration::from_secs(endpoint.timeout))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    self.update_service_status(service_name, ServiceStatus::Healthy).await;
                    debug!("Health check passed for service: {}", service_name);
                    Ok(())
                } else {
                    self.update_service_status(service_name, ServiceStatus::Unhealthy).await;
                    Err(anyhow::anyhow!("Health check failed with status: {}", response.status()))
                }
            }
            Err(e) => {
                self.update_service_status(service_name, ServiceStatus::Unhealthy).await;
                Err(anyhow::anyhow!("Health check request failed: {}", e))
            }
        }
    }

    /// 更新服务状态
    pub async fn update_service_status(&self, name: &str, status: ServiceStatus) {
        let mut services = self.services.write().await;
        
        if let Some(service) = services.get_mut(name) {
            let old_status = service.status.clone();
            service.status = status.clone();
            service.last_health_check = Some(Instant::now());
            
            if old_status != status {
                info!("Service {} status changed from {:?} to {:?}", name, old_status, status);
            }
        } else {
            // 如果服务不存在，从配置中创建
            if let Some(endpoint) = self.config.get_service_endpoint(name) {
                let service_info = ServiceInfo {
                    name: name.to_string(),
                    url: endpoint.url.clone(),
                    status,
                    version: "unknown".to_string(),
                    metadata: HashMap::new(),
                    registered_at: Instant::now(),
                    last_health_check: Some(Instant::now()),
                };
                services.insert(name.to_string(), service_info);
            }
        }
    }

    /// 启动定期健康检查
    pub async fn start_health_checks(&self, interval: Duration) {
        let registry = self.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                registry.perform_health_checks().await;
            }
        });
        
        info!("Health check scheduler started with interval: {:?}", interval);
    }

    /// 执行所有服务的健康检查
    async fn perform_health_checks(&self) {
        let services = ["user", "trading", "market-data", "strategy", "risk", "notification", "analytics"];
        
        for service_name in services {
            if let Err(e) = self.health_check(service_name).await {
                warn!("Health check failed for {}: {}", service_name, e);
            }
        }
    }

    /// 服务发现
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>> {
        // 这里可以集成服务发现机制，如Consul、etcd等
        // 目前从配置中加载服务
        let mut discovered_services = Vec::new();
        
        let service_configs = [
            ("user", &self.config.services.user_service),
            ("trading", &self.config.services.trading_service),
            ("market-data", &self.config.services.market_data_service),
            ("strategy", &self.config.services.strategy_service),
            ("risk", &self.config.services.risk_service),
            ("notification", &self.config.services.notification_service),
            ("analytics", &self.config.services.analytics_service),
        ];

        for (name, endpoint) in service_configs {
            let service_info = ServiceInfo {
                name: name.to_string(),
                url: endpoint.url.clone(),
                status: ServiceStatus::Unknown,
                version: "unknown".to_string(),
                metadata: HashMap::new(),
                registered_at: Instant::now(),
                last_health_check: None,
            };
            
            discovered_services.push(service_info.clone());
            self.register_service(name.to_string(), service_info).await;
        }

        info!("Discovered {} services", discovered_services.len());
        Ok(discovered_services)
    }

    /// 负载均衡 - 轮询算法
    pub async fn round_robin_select(&self, service_name: &str) -> Option<ServiceInfo> {
        // 简化实现，实际应该维护每个服务的实例列表和轮询状态
        self.get_healthy_service(service_name).await
    }

    /// 负载均衡 - 随机算法
    pub async fn random_select(&self, service_name: &str) -> Option<ServiceInfo> {
        // 简化实现
        self.get_healthy_service(service_name).await
    }

    /// 负载均衡 - 最少连接算法
    pub async fn least_connections_select(&self, service_name: &str) -> Option<ServiceInfo> {
        // 简化实现
        self.get_healthy_service(service_name).await
    }

    /// 获取服务统计信息
    pub async fn get_service_stats(&self) -> ServiceRegistryStats {
        let services = self.services.read().await;
        let total_services = services.len();
        let healthy_services = services.values()
            .filter(|s| s.status == ServiceStatus::Healthy)
            .count();
        let unhealthy_services = services.values()
            .filter(|s| s.status == ServiceStatus::Unhealthy)
            .count();
        let unknown_services = services.values()
            .filter(|s| s.status == ServiceStatus::Unknown)
            .count();

        ServiceRegistryStats {
            total_services,
            healthy_services,
            unhealthy_services,
            unknown_services,
        }
    }
}

/// 服务信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub url: String,
    pub status: ServiceStatus,
    pub version: String,
    pub metadata: HashMap<String, String>,
    #[serde(skip)]
    pub registered_at: Instant,
    #[serde(skip)]
    pub last_health_check: Option<Instant>,
}

/// 服务状态
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// 服务注册表统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceRegistryStats {
    pub total_services: usize,
    pub healthy_services: usize,
    pub unhealthy_services: usize,
    pub unknown_services: usize,
}

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
    WeightedRoundRobin,
    ConsistentHash,
}

/// 服务发现配置
#[derive(Debug, Clone)]
pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub provider: ServiceDiscoveryProvider,
    pub refresh_interval: Duration,
    pub health_check_interval: Duration,
}

/// 服务发现提供者
#[derive(Debug, Clone)]
pub enum ServiceDiscoveryProvider {
    Static,
    Consul,
    Etcd,
    Kubernetes,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: ServiceDiscoveryProvider::Static,
            refresh_interval: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registration() {
        let config = GatewayConfig::default();
        let registry = ServiceRegistry::new(config);
        
        let service_info = ServiceInfo {
            name: "test-service".to_string(),
            url: "http://localhost:8080".to_string(),
            status: ServiceStatus::Healthy,
            version: "1.0.0".to_string(),
            metadata: HashMap::new(),
            registered_at: Instant::now(),
            last_health_check: None,
        };

        registry.register_service("test-service".to_string(), service_info.clone()).await;
        
        let retrieved = registry.get_service("test-service").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-service");
    }

    #[tokio::test]
    async fn test_service_status_update() {
        let config = GatewayConfig::default();
        let registry = ServiceRegistry::new(config);
        
        let service_info = ServiceInfo {
            name: "test-service".to_string(),
            url: "http://localhost:8080".to_string(),
            status: ServiceStatus::Healthy,
            version: "1.0.0".to_string(),
            metadata: HashMap::new(),
            registered_at: Instant::now(),
            last_health_check: None,
        };

        registry.register_service("test-service".to_string(), service_info).await;
        registry.update_service_status("test-service", ServiceStatus::Unhealthy).await;
        
        let service = registry.get_service("test-service").await.unwrap();
        assert_eq!(service.status, ServiceStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_healthy_service_selection() {
        let config = GatewayConfig::default();
        let registry = ServiceRegistry::new(config);
        
        // 注册健康服务
        let healthy_service = ServiceInfo {
            name: "healthy-service".to_string(),
            url: "http://localhost:8080".to_string(),
            status: ServiceStatus::Healthy,
            version: "1.0.0".to_string(),
            metadata: HashMap::new(),
            registered_at: Instant::now(),
            last_health_check: None,
        };

        // 注册不健康服务
        let unhealthy_service = ServiceInfo {
            name: "unhealthy-service".to_string(),
            url: "http://localhost:8081".to_string(),
            status: ServiceStatus::Unhealthy,
            version: "1.0.0".to_string(),
            metadata: HashMap::new(),
            registered_at: Instant::now(),
            last_health_check: None,
        };

        registry.register_service("healthy-service".to_string(), healthy_service).await;
        registry.register_service("unhealthy-service".to_string(), unhealthy_service).await;
        
        // 应该只返回健康的服务
        let healthy = registry.get_healthy_service("healthy-service").await;
        assert!(healthy.is_some());
        
        let unhealthy = registry.get_healthy_service("unhealthy-service").await;
        assert!(unhealthy.is_none());
    }
}