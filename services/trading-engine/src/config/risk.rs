use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 风险管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub enabled: bool,
    pub max_position_size: Decimal,
    pub max_daily_loss: Decimal,
    pub max_leverage: Decimal,
    pub margin_call_threshold: Decimal,
    pub liquidation_threshold: Decimal,
    pub position_limits: PositionLimits,
    pub trading_limits: TradingLimits,
    pub risk_checks: RiskChecks,
}

/// 仓位限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLimits {
    pub max_positions_per_user: u32,
    pub max_position_value: Decimal,
    pub max_concentration: Decimal,        // 单一资产最大占比
    pub max_correlation_exposure: Decimal, // 相关性资产最大敞口
}

/// 交易限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingLimits {
    pub max_orders_per_second: u32,
    pub max_orders_per_minute: u32,
    pub max_orders_per_hour: u32,
    pub max_daily_volume: Decimal,
    pub max_order_frequency: Duration,
    pub cooling_period: Duration,
}

/// 风险检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskChecks {
    pub pre_trade_checks: PreTradeChecks,
    pub post_trade_checks: PostTradeChecks,
    pub real_time_monitoring: RealTimeMonitoring,
}

/// 交易前检查
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreTradeChecks {
    pub balance_check: bool,
    pub position_limit_check: bool,
    pub leverage_check: bool,
    pub concentration_check: bool,
    pub correlation_check: bool,
    pub market_impact_check: bool,
}

/// 交易后检查
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTradeChecks {
    pub pnl_check: bool,
    pub margin_check: bool,
    pub exposure_check: bool,
    pub var_check: bool, // Value at Risk
}

/// 实时监控
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMonitoring {
    pub enabled: bool,
    pub check_interval: Duration,
    pub alert_thresholds: AlertThresholds,
    pub auto_actions: AutoActions,
}

/// 告警阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub margin_warning: Decimal,    // 保证金预警阈值
    pub margin_critical: Decimal,   // 保证金危险阈值
    pub loss_warning: Decimal,      // 亏损预警阈值
    pub loss_critical: Decimal,     // 亏损危险阈值
    pub exposure_warning: Decimal,  // 敞口预警阈值
    pub exposure_critical: Decimal, // 敞口危险阈值
}

/// 自动操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoActions {
    pub auto_margin_call: bool,
    pub auto_liquidation: bool,
    pub auto_position_reduction: bool,
    pub auto_trading_halt: bool,
    pub emergency_stop: bool,
}

impl RiskConfig {
    /// 验证风险配置
    pub fn validate(&self) -> Result<()> {
        if self.max_position_size <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Max position size must be positive"));
        }

        if self.max_daily_loss <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Max daily loss must be positive"));
        }

        if self.max_leverage <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Max leverage must be positive"));
        }

        if self.margin_call_threshold <= Decimal::ZERO || self.margin_call_threshold >= Decimal::ONE
        {
            return Err(anyhow::anyhow!(
                "Margin call threshold must be between 0 and 1"
            ));
        }

        if self.liquidation_threshold <= Decimal::ZERO || self.liquidation_threshold >= Decimal::ONE
        {
            return Err(anyhow::anyhow!(
                "Liquidation threshold must be between 0 and 1"
            ));
        }

        if self.margin_call_threshold >= self.liquidation_threshold {
            return Err(anyhow::anyhow!(
                "Margin call threshold must be less than liquidation threshold"
            ));
        }

        // 验证子配置
        self.position_limits.validate()?;
        self.trading_limits.validate()?;

        Ok(())
    }

    /// 检查是否需要保证金追加
    pub fn requires_margin_call(&self, margin_ratio: Decimal) -> bool {
        margin_ratio <= self.margin_call_threshold
    }

    /// 检查是否需要强制平仓
    pub fn requires_liquidation(&self, margin_ratio: Decimal) -> bool {
        margin_ratio <= self.liquidation_threshold
    }

    /// 检查仓位大小是否超限
    pub fn is_position_size_valid(&self, size: Decimal) -> bool {
        size <= self.max_position_size
    }

    /// 检查杠杆是否超限
    pub fn is_leverage_valid(&self, leverage: Decimal) -> bool {
        leverage <= self.max_leverage
    }
}

impl PositionLimits {
    /// 验证仓位限制
    pub fn validate(&self) -> Result<()> {
        if self.max_positions_per_user == 0 {
            return Err(anyhow::anyhow!("Max positions per user cannot be 0"));
        }

        if self.max_position_value <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Max position value must be positive"));
        }

        if self.max_concentration <= Decimal::ZERO || self.max_concentration > Decimal::ONE {
            return Err(anyhow::anyhow!("Max concentration must be between 0 and 1"));
        }

        if self.max_correlation_exposure <= Decimal::ZERO
            || self.max_correlation_exposure > Decimal::ONE
        {
            return Err(anyhow::anyhow!(
                "Max correlation exposure must be between 0 and 1"
            ));
        }

        Ok(())
    }

    /// 检查仓位数量是否超限
    pub fn is_position_count_valid(&self, count: u32) -> bool {
        count <= self.max_positions_per_user
    }

    /// 检查仓位价值是否超限
    pub fn is_position_value_valid(&self, value: Decimal) -> bool {
        value <= self.max_position_value
    }

    /// 检查集中度是否超限
    pub fn is_concentration_valid(&self, concentration: Decimal) -> bool {
        concentration <= self.max_concentration
    }
}

impl TradingLimits {
    /// 验证交易限制
    pub fn validate(&self) -> Result<()> {
        if self.max_orders_per_second == 0 {
            return Err(anyhow::anyhow!("Max orders per second cannot be 0"));
        }

        if self.max_orders_per_minute == 0 {
            return Err(anyhow::anyhow!("Max orders per minute cannot be 0"));
        }

        if self.max_orders_per_hour == 0 {
            return Err(anyhow::anyhow!("Max orders per hour cannot be 0"));
        }

        if self.max_daily_volume <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Max daily volume must be positive"));
        }

        Ok(())
    }

    /// 检查订单频率是否超限
    pub fn is_order_frequency_valid(&self, last_order_time: chrono::DateTime<chrono::Utc>) -> bool {
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(last_order_time);
        elapsed.to_std().unwrap_or(Duration::ZERO) >= self.max_order_frequency
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_position_size: Decimal::from(10_000_000),
            max_daily_loss: Decimal::from(100_000),
            max_leverage: Decimal::from(10),
            margin_call_threshold: Decimal::new(8, 1), // 0.8
            liquidation_threshold: Decimal::new(9, 1), // 0.9
            position_limits: PositionLimits::default(),
            trading_limits: TradingLimits::default(),
            risk_checks: RiskChecks::default(),
        }
    }
}

impl Default for PositionLimits {
    fn default() -> Self {
        Self {
            max_positions_per_user: 50,
            max_position_value: Decimal::from(5_000_000),
            max_concentration: Decimal::new(3, 1), // 0.3 (30%)
            max_correlation_exposure: Decimal::new(5, 1), // 0.5 (50%)
        }
    }
}

impl Default for TradingLimits {
    fn default() -> Self {
        Self {
            max_orders_per_second: 10,
            max_orders_per_minute: 100,
            max_orders_per_hour: 1000,
            max_daily_volume: Decimal::from(10_000_000),
            max_order_frequency: Duration::from_millis(100), // 100ms
            cooling_period: Duration::from_secs(60),         // 1 minute
        }
    }
}

impl Default for RiskChecks {
    fn default() -> Self {
        Self {
            pre_trade_checks: PreTradeChecks::default(),
            post_trade_checks: PostTradeChecks::default(),
            real_time_monitoring: RealTimeMonitoring::default(),
        }
    }
}

impl Default for PreTradeChecks {
    fn default() -> Self {
        Self {
            balance_check: true,
            position_limit_check: true,
            leverage_check: true,
            concentration_check: true,
            correlation_check: true,
            market_impact_check: true,
        }
    }
}

impl Default for PostTradeChecks {
    fn default() -> Self {
        Self {
            pnl_check: true,
            margin_check: true,
            exposure_check: true,
            var_check: true,
        }
    }
}

impl Default for RealTimeMonitoring {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(1), // 1 second
            alert_thresholds: AlertThresholds::default(),
            auto_actions: AutoActions::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            margin_warning: Decimal::new(85, 2),   // 0.85
            margin_critical: Decimal::new(9, 1),   // 0.9
            loss_warning: Decimal::new(5, 2),      // 0.05 (5%)
            loss_critical: Decimal::new(1, 1),     // 0.1 (10%)
            exposure_warning: Decimal::new(7, 1),  // 0.7 (70%)
            exposure_critical: Decimal::new(9, 1), // 0.9 (90%)
        }
    }
}

impl Default for AutoActions {
    fn default() -> Self {
        Self {
            auto_margin_call: true,
            auto_liquidation: true,
            auto_position_reduction: true,
            auto_trading_halt: false,
            emergency_stop: true,
        }
    }
}
