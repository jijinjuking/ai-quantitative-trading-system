use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 执行引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub enabled: bool,
    pub max_slippage: Decimal,
    pub execution_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
    pub algorithms: AlgorithmConfig,
    pub routing: RoutingConfig,
    pub latency: LatencyConfig,
}

/// 算法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    pub twap: TwapConfig,
    pub vwap: VwapConfig,
    pub iceberg: IcebergConfig,
    pub pov: PovConfig, // Percentage of Volume
}

/// TWAP (Time Weighted Average Price) 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwapConfig {
    pub enabled: bool,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub slice_interval: Duration,
    pub max_participation_rate: Decimal,
}

/// VWAP (Volume Weighted Average Price) 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VwapConfig {
    pub enabled: bool,
    pub lookback_period: Duration,
    pub max_participation_rate: Decimal,
    pub volume_curve_adjustment: bool,
}

/// 冰山订单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergConfig {
    pub enabled: bool,
    pub min_visible_size: Decimal,
    pub max_visible_size: Decimal,
    pub refresh_threshold: Decimal,
    pub randomization: bool,
}

/// POV (Percentage of Volume) 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PovConfig {
    pub enabled: bool,
    pub min_participation_rate: Decimal,
    pub max_participation_rate: Decimal,
    pub volume_lookback: Duration,
    pub aggressive_mode: bool,
}

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub enabled: bool,
    pub venues: Vec<VenueConfig>,
    pub routing_strategy: RoutingStrategy,
    pub smart_routing: SmartRoutingConfig,
}

/// 交易所配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueConfig {
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub max_order_size: Decimal,
    pub min_order_size: Decimal,
    pub fee_rate: Decimal,
    pub latency_ms: u64,
    pub reliability_score: Decimal,
}

/// 路由策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    BestPrice,
    LowestFee,
    FastestExecution,
    SmartRouting,
    RoundRobin,
}

/// 智能路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRoutingConfig {
    pub enabled: bool,
    pub price_improvement_threshold: Decimal,
    pub latency_weight: Decimal,
    pub fee_weight: Decimal,
    pub liquidity_weight: Decimal,
    pub reliability_weight: Decimal,
}

/// 延迟配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    pub target_latency: Duration,
    pub max_acceptable_latency: Duration,
    pub latency_monitoring: bool,
    pub latency_alerts: bool,
    pub performance_optimization: PerformanceConfig,
}

/// 性能优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub connection_pooling: bool,
    pub request_batching: bool,
    pub async_processing: bool,
    pub cache_market_data: bool,
    pub precompute_calculations: bool,
}

impl ExecutionConfig {
    /// 验证执行配置
    pub fn validate(&self) -> Result<()> {
        if self.max_slippage < Decimal::ZERO || self.max_slippage > Decimal::ONE {
            return Err(anyhow::anyhow!("Max slippage must be between 0 and 1"));
        }

        if self.retry_attempts == 0 {
            return Err(anyhow::anyhow!("Retry attempts cannot be 0"));
        }

        // 验证算法配置
        self.algorithms.validate()?;
        self.routing.validate()?;
        self.latency.validate()?;

        Ok(())
    }

    /// 检查滑点是否可接受
    pub fn is_slippage_acceptable(&self, slippage: Decimal) -> bool {
        slippage.abs() <= self.max_slippage
    }

    /// 获取重试延迟
    pub fn get_retry_delay(&self, attempt: u32) -> Duration {
        // 指数退避策略
        let multiplier = 2_u64.pow(attempt.min(10)); // 最大1024倍
        Duration::from_millis(self.retry_delay.as_millis() as u64 * multiplier)
    }
}

impl AlgorithmConfig {
    /// 验证算法配置
    pub fn validate(&self) -> Result<()> {
        self.twap.validate()?;
        self.vwap.validate()?;
        self.iceberg.validate()?;
        self.pov.validate()?;
        Ok(())
    }
}

impl TwapConfig {
    /// 验证TWAP配置
    pub fn validate(&self) -> Result<()> {
        if self.min_duration >= self.max_duration {
            return Err(anyhow::anyhow!(
                "TWAP min duration must be less than max duration"
            ));
        }

        if self.max_participation_rate <= Decimal::ZERO
            || self.max_participation_rate > Decimal::ONE
        {
            return Err(anyhow::anyhow!(
                "TWAP max participation rate must be between 0 and 1"
            ));
        }

        Ok(())
    }
}

impl VwapConfig {
    /// 验证VWAP配置
    pub fn validate(&self) -> Result<()> {
        if self.max_participation_rate <= Decimal::ZERO
            || self.max_participation_rate > Decimal::ONE
        {
            return Err(anyhow::anyhow!(
                "VWAP max participation rate must be between 0 and 1"
            ));
        }

        Ok(())
    }
}

impl IcebergConfig {
    /// 验证冰山订单配置
    pub fn validate(&self) -> Result<()> {
        if self.min_visible_size >= self.max_visible_size {
            return Err(anyhow::anyhow!(
                "Iceberg min visible size must be less than max visible size"
            ));
        }

        if self.refresh_threshold <= Decimal::ZERO || self.refresh_threshold > Decimal::ONE {
            return Err(anyhow::anyhow!(
                "Iceberg refresh threshold must be between 0 and 1"
            ));
        }

        Ok(())
    }
}

impl PovConfig {
    /// 验证POV配置
    pub fn validate(&self) -> Result<()> {
        if self.min_participation_rate >= self.max_participation_rate {
            return Err(anyhow::anyhow!(
                "POV min participation rate must be less than max participation rate"
            ));
        }

        if self.max_participation_rate > Decimal::ONE {
            return Err(anyhow::anyhow!(
                "POV max participation rate cannot exceed 1"
            ));
        }

        Ok(())
    }
}

impl RoutingConfig {
    /// 验证路由配置
    pub fn validate(&self) -> Result<()> {
        if self.venues.is_empty() {
            return Err(anyhow::anyhow!("At least one venue must be configured"));
        }

        for venue in &self.venues {
            venue.validate()?;
        }

        self.smart_routing.validate()?;

        Ok(())
    }

    /// 获取启用的交易所
    pub fn get_enabled_venues(&self) -> Vec<&VenueConfig> {
        self.venues.iter().filter(|v| v.enabled).collect()
    }

    /// 根据优先级排序交易所
    pub fn get_venues_by_priority(&self) -> Vec<&VenueConfig> {
        let mut venues: Vec<&VenueConfig> = self.get_enabled_venues();
        venues.sort_by_key(|v| v.priority);
        venues
    }
}

impl VenueConfig {
    /// 验证交易所配置
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Venue name is required"));
        }

        if self.min_order_size >= self.max_order_size {
            return Err(anyhow::anyhow!(
                "Venue min order size must be less than max order size"
            ));
        }

        if self.fee_rate < Decimal::ZERO {
            return Err(anyhow::anyhow!("Venue fee rate cannot be negative"));
        }

        if self.reliability_score < Decimal::ZERO || self.reliability_score > Decimal::ONE {
            return Err(anyhow::anyhow!(
                "Venue reliability score must be between 0 and 1"
            ));
        }

        Ok(())
    }

    /// 检查订单大小是否在范围内
    pub fn is_order_size_valid(&self, size: Decimal) -> bool {
        size >= self.min_order_size && size <= self.max_order_size
    }
}

impl SmartRoutingConfig {
    /// 验证智能路由配置
    pub fn validate(&self) -> Result<()> {
        let total_weight =
            self.latency_weight + self.fee_weight + self.liquidity_weight + self.reliability_weight;

        if (total_weight - Decimal::ONE).abs() > Decimal::new(1, 6) {
            return Err(anyhow::anyhow!("Smart routing weights must sum to 1.0"));
        }

        Ok(())
    }
}

impl LatencyConfig {
    /// 验证延迟配置
    pub fn validate(&self) -> Result<()> {
        if self.target_latency >= self.max_acceptable_latency {
            return Err(anyhow::anyhow!(
                "Target latency must be less than max acceptable latency"
            ));
        }

        Ok(())
    }

    /// 检查延迟是否可接受
    pub fn is_latency_acceptable(&self, latency: Duration) -> bool {
        latency <= self.max_acceptable_latency
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_slippage: Decimal::new(1, 2), // 0.01 (1%)
            execution_timeout: Duration::from_secs(5),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(1000),
            algorithms: AlgorithmConfig::default(),
            routing: RoutingConfig::default(),
            latency: LatencyConfig::default(),
        }
    }
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            twap: TwapConfig::default(),
            vwap: VwapConfig::default(),
            iceberg: IcebergConfig::default(),
            pov: PovConfig::default(),
        }
    }
}

impl Default for TwapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_duration: Duration::from_secs(60),
            max_duration: Duration::from_secs(3600),
            slice_interval: Duration::from_secs(30),
            max_participation_rate: Decimal::new(2, 1), // 0.2 (20%)
        }
    }
}

impl Default for VwapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lookback_period: Duration::from_secs(3600),
            max_participation_rate: Decimal::new(3, 1), // 0.3 (30%)
            volume_curve_adjustment: true,
        }
    }
}

impl Default for IcebergConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_visible_size: Decimal::new(1, 2),  // 0.01
            max_visible_size: Decimal::new(1, 1),  // 0.1
            refresh_threshold: Decimal::new(1, 1), // 0.1 (10%)
            randomization: true,
        }
    }
}

impl Default for PovConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_participation_rate: Decimal::new(5, 2), // 0.05 (5%)
            max_participation_rate: Decimal::new(25, 2), // 0.25 (25%)
            volume_lookback: Duration::from_secs(300),
            aggressive_mode: false,
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            venues: vec![
                VenueConfig {
                    name: "Binance".to_string(),
                    enabled: true,
                    priority: 1,
                    max_order_size: Decimal::from(1_000_000),
                    min_order_size: Decimal::new(1, 3), // 0.001
                    fee_rate: Decimal::new(1, 3),       // 0.1%
                    latency_ms: 50,
                    reliability_score: Decimal::new(95, 2), // 0.95
                },
                VenueConfig {
                    name: "Coinbase".to_string(),
                    enabled: true,
                    priority: 2,
                    max_order_size: Decimal::from(500_000),
                    min_order_size: Decimal::new(1, 3), // 0.001
                    fee_rate: Decimal::new(15, 4),      // 0.15%
                    latency_ms: 75,
                    reliability_score: Decimal::new(92, 2), // 0.92
                },
            ],
            routing_strategy: RoutingStrategy::SmartRouting,
            smart_routing: SmartRoutingConfig::default(),
        }
    }
}

impl Default for SmartRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            price_improvement_threshold: Decimal::new(1, 4), // 0.0001
            latency_weight: Decimal::new(3, 1),              // 0.3
            fee_weight: Decimal::new(25, 2),                 // 0.25
            liquidity_weight: Decimal::new(25, 2),           // 0.25
            reliability_weight: Decimal::new(2, 1),          // 0.2
        }
    }
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            target_latency: Duration::from_millis(10),
            max_acceptable_latency: Duration::from_millis(100),
            latency_monitoring: true,
            latency_alerts: true,
            performance_optimization: PerformanceConfig::default(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            connection_pooling: true,
            request_batching: true,
            async_processing: true,
            cache_market_data: true,
            precompute_calculations: true,
        }
    }
}
