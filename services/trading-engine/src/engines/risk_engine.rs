use anyhow::Result;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    config::TradingEngineConfig,
    models::{Order, Position, Symbol, TradingError, TradingResult},
};

/// 专业级风险管理引擎
/// 实时监控、多层风控、智能预警
#[derive(Clone)]
pub struct RiskEngine {
    config: TradingEngineConfig,
    /// 用户风险配置
    user_risk_configs: Arc<RwLock<HashMap<Uuid, UserRiskConfig>>>,
    /// 系统风险限制
    system_limits: Arc<RwLock<SystemRiskLimits>>,
    /// 实时风险监控
    risk_monitor: Arc<RwLock<RiskMonitor>>,
    /// 风险事件历史
    risk_events: Arc<RwLock<Vec<RiskEvent>>>,
}

#[derive(Debug, Clone)]
pub struct UserRiskConfig {
    pub user_id: Uuid,
    pub max_position_value: Decimal,
    pub max_daily_loss: Decimal,
    pub max_leverage: Decimal,
    pub allowed_symbols: Option<Vec<Symbol>>,
    pub blocked_symbols: Vec<Symbol>,
    pub max_order_value: Decimal,
    pub max_orders_per_minute: u32,
    pub margin_call_threshold: Decimal,
    pub liquidation_threshold: Decimal,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SystemRiskLimits {
    pub max_total_exposure: Decimal,
    pub max_symbol_concentration: Decimal, // 单个交易对最大敞口比例
    pub max_user_concentration: Decimal,   // 单个用户最大敞口比例
    pub circuit_breaker_threshold: Decimal, // 熔断阈值
    pub max_drawdown_threshold: Decimal,   // 最大回撤阈值
    pub volatility_threshold: Decimal,     // 波动率阈值
}

#[derive(Debug, Clone)]
pub struct RiskMonitor {
    pub total_exposure: Decimal,
    pub symbol_exposures: HashMap<Symbol, Decimal>,
    pub user_exposures: HashMap<Uuid, Decimal>,
    pub daily_pnl: Decimal,
    pub max_drawdown: Decimal,
    pub current_volatility: HashMap<Symbol, Decimal>,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct RiskEvent {
    pub event_id: Uuid,
    pub event_type: RiskEventType,
    pub user_id: Option<Uuid>,
    pub symbol: Option<Symbol>,
    pub severity: RiskSeverity,
    pub message: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskEventType {
    PositionLimitExceeded,
    MarginCall,
    Liquidation,
    DailyLossLimitExceeded,
    OrderRateExceeded,
    SystemExposureExceeded,
    CircuitBreaker,
    VolatilitySpike,
    ConcentrationRisk,
    SuspiciousActivity,
}

impl std::fmt::Display for RiskEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskEventType::PositionLimitExceeded => write!(f, "POSITION_LIMIT_EXCEEDED"),
            RiskEventType::MarginCall => write!(f, "MARGIN_CALL"),
            RiskEventType::Liquidation => write!(f, "LIQUIDATION"),
            RiskEventType::DailyLossLimitExceeded => write!(f, "DAILY_LOSS_LIMIT_EXCEEDED"),
            RiskEventType::OrderRateExceeded => write!(f, "ORDER_RATE_EXCEEDED"),
            RiskEventType::SystemExposureExceeded => write!(f, "SYSTEM_EXPOSURE_EXCEEDED"),
            RiskEventType::CircuitBreaker => write!(f, "CIRCUIT_BREAKER"),
            RiskEventType::VolatilitySpike => write!(f, "VOLATILITY_SPIKE"),
            RiskEventType::ConcentrationRisk => write!(f, "CONCENTRATION_RISK"),
            RiskEventType::SuspiciousActivity => write!(f, "SUSPICIOUS_ACTIVITY"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
    pub max_allowed_size: Option<Decimal>,
    pub required_margin: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: RiskSeverity,
    pub value: Decimal,
    pub threshold: Decimal,
    pub description: String,
}

impl RiskEngine {
    pub fn new(config: TradingEngineConfig) -> Self {
        let system_limits = SystemRiskLimits {
            max_total_exposure: Decimal::from(10_000_000), // 1000万
            max_symbol_concentration: Decimal::new(20, 2), // 20%
            max_user_concentration: Decimal::new(10, 2),   // 10%
            circuit_breaker_threshold: Decimal::new(5, 2), // 5%
            max_drawdown_threshold: Decimal::new(15, 2),   // 15%
            volatility_threshold: Decimal::new(50, 2),     // 50%
        };

        let risk_monitor = RiskMonitor {
            total_exposure: Decimal::ZERO,
            symbol_exposures: HashMap::new(),
            user_exposures: HashMap::new(),
            daily_pnl: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            current_volatility: HashMap::new(),
            last_update: chrono::Utc::now(),
        };

        Self {
            config,
            user_risk_configs: Arc::new(RwLock::new(HashMap::new())),
            system_limits: Arc::new(RwLock::new(system_limits)),
            risk_monitor: Arc::new(RwLock::new(risk_monitor)),
            risk_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 设置用户风险配置
    pub async fn set_user_risk_config(&self, config: UserRiskConfig) {
        let mut configs = self.user_risk_configs.write().await;
        configs.insert(config.user_id, config);
    }

    /// 获取用户风险配置
    pub async fn get_user_risk_config(&self, user_id: Uuid) -> Option<UserRiskConfig> {
        let configs = self.user_risk_configs.read().await;
        configs.get(&user_id).cloned()
    }

    /// 订单前风险检查
    pub async fn validate_order(&self, order: &Order) -> TradingResult<RiskAssessment> {
        let user_config = self.get_user_risk_config(order.user_id).await
            .ok_or_else(|| TradingError::RiskViolation("User risk config not found".to_string()))?;

        if !user_config.is_active {
            return Err(TradingError::RiskViolation("User trading is suspended".to_string()));
        }

        let mut risk_factors = Vec::new();
        let mut recommendations = Vec::new();

        // 1. 检查交易对限制
        self.check_symbol_restrictions(order, &user_config, &mut risk_factors)?;

        // 2. 检查订单价值限制
        self.check_order_value_limit(order, &user_config, &mut risk_factors)?;

        // 3. 检查仓位限制
        self.check_position_limits(order, &user_config, &mut risk_factors).await?;

        // 4. 检查保证金要求
        let required_margin = self.calculate_margin_requirement(order).await?;
        self.check_margin_sufficiency(order.user_id, required_margin, &mut risk_factors).await?;

        // 5. 检查交易频率
        self.check_trading_frequency(order.user_id, &user_config, &mut risk_factors).await?;

        // 6. 检查系统级风险
        self.check_system_risk(order, &mut risk_factors).await?;

        // 7. 检查市场风险
        self.check_market_risk(order, &mut risk_factors).await?;

        // 评估整体风险等级
        let overall_risk = self.assess_overall_risk(&risk_factors);

        // 生成建议
        self.generate_recommendations(&risk_factors, &mut recommendations);

        // 计算最大允许交易量
        let max_allowed_size = self.calculate_max_allowed_size(order, &user_config).await?;

        Ok(RiskAssessment {
            overall_risk,
            risk_factors,
            recommendations,
            max_allowed_size: Some(max_allowed_size),
            required_margin: Some(required_margin),
        })
    }

    /// 检查交易对限制
    fn check_symbol_restrictions(
        &self,
        order: &Order,
        config: &UserRiskConfig,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        // 检查黑名单
        if config.blocked_symbols.contains(&order.symbol) {
            return Err(TradingError::RiskViolation(format!(
                "Symbol {} is blocked for user {}",
                order.symbol, order.user_id
            )));
        }

        // 检查白名单
        if let Some(ref allowed) = config.allowed_symbols {
            if !allowed.contains(&order.symbol) {
                return Err(TradingError::RiskViolation(format!(
                    "Symbol {} is not in allowed list for user {}",
                    order.symbol, order.user_id
                )));
            }
        }

        Ok(())
    }

    /// 检查订单价值限制
    fn check_order_value_limit(
        &self,
        order: &Order,
        config: &UserRiskConfig,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        if let Some(order_value) = order.calculate_value() {
            if order_value > config.max_order_value {
                risk_factors.push(RiskFactor {
                    factor_type: "ORDER_VALUE_LIMIT".to_string(),
                    severity: RiskSeverity::High,
                    value: order_value,
                    threshold: config.max_order_value,
                    description: "Order value exceeds limit".to_string(),
                });

                return Err(TradingError::RiskViolation(format!(
                    "Order value {} exceeds limit {}",
                    order_value, config.max_order_value
                )));
            }

            // 警告阈值（80%）
            let warning_threshold = config.max_order_value * Decimal::new(8, 1);
            if order_value > warning_threshold {
                risk_factors.push(RiskFactor {
                    factor_type: "ORDER_VALUE_WARNING".to_string(),
                    severity: RiskSeverity::Medium,
                    value: order_value,
                    threshold: warning_threshold,
                    description: "Order value approaching limit".to_string(),
                });
            }
        }

        Ok(())
    }

    /// 检查仓位限制
    async fn check_position_limits(
        &self,
        order: &Order,
        config: &UserRiskConfig,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        // TODO: 从仓位服务获取当前仓位
        let current_position_value = Decimal::ZERO; // 临时值

        let order_value = order.calculate_value().unwrap_or(Decimal::ZERO);
        let total_position_value = current_position_value + order_value;

        if total_position_value > config.max_position_value {
            risk_factors.push(RiskFactor {
                factor_type: "POSITION_LIMIT".to_string(),
                severity: RiskSeverity::High,
                value: total_position_value,
                threshold: config.max_position_value,
                description: "Total position value exceeds limit".to_string(),
            });

            return Err(TradingError::RiskViolation(format!(
                "Total position value {} exceeds limit {}",
                total_position_value, config.max_position_value
            )));
        }

        Ok(())
    }

    /// 计算保证金要求
    async fn calculate_margin_requirement(&self, order: &Order) -> TradingResult<Decimal> {
        // 简化计算，实际应该根据交易对、杠杆等因素
        let order_value = order.calculate_value().unwrap_or(Decimal::ZERO);
        let leverage = Decimal::from(10); // 默认10倍杠杆
        Ok(order_value / leverage)
    }

    /// 检查保证金充足性
    async fn check_margin_sufficiency(
        &self,
        user_id: Uuid,
        required_margin: Decimal,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        // TODO: 从账户服务获取可用保证金
        let available_margin = Decimal::from(100000); // 临时值

        if available_margin < required_margin {
            risk_factors.push(RiskFactor {
                factor_type: "INSUFFICIENT_MARGIN".to_string(),
                severity: RiskSeverity::Critical,
                value: available_margin,
                threshold: required_margin,
                description: "Insufficient margin for order".to_string(),
            });

            return Err(TradingError::InsufficientMargin {
                required: required_margin,
                available: available_margin,
            });
        }

        Ok(())
    }

    /// 检查交易频率
    async fn check_trading_frequency(
        &self,
        user_id: Uuid,
        config: &UserRiskConfig,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        // TODO: 实现交易频率检查
        // 统计最近1分钟的订单数量
        let recent_orders = 0; // 临时值

        if recent_orders >= config.max_orders_per_minute {
            risk_factors.push(RiskFactor {
                factor_type: "ORDER_RATE_LIMIT".to_string(),
                severity: RiskSeverity::High,
                value: Decimal::from(recent_orders),
                threshold: Decimal::from(config.max_orders_per_minute),
                description: "Order rate limit exceeded".to_string(),
            });

            return Err(TradingError::RiskViolation(
                "Order rate limit exceeded".to_string(),
            ));
        }

        Ok(())
    }

    /// 检查系统级风险
    async fn check_system_risk(
        &self,
        order: &Order,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        let monitor = self.risk_monitor.read().await;
        let limits = self.system_limits.read().await;

        // 检查总敞口
        let order_value = order.calculate_value().unwrap_or(Decimal::ZERO);
        let new_total_exposure = monitor.total_exposure + order_value;

        if new_total_exposure > limits.max_total_exposure {
            risk_factors.push(RiskFactor {
                factor_type: "SYSTEM_EXPOSURE_LIMIT".to_string(),
                severity: RiskSeverity::Critical,
                value: new_total_exposure,
                threshold: limits.max_total_exposure,
                description: "System exposure limit exceeded".to_string(),
            });

            return Err(TradingError::RiskViolation(
                "System exposure limit exceeded".to_string(),
            ));
        }

        // 检查交易对集中度
        let symbol_exposure = monitor.symbol_exposures.get(&order.symbol).unwrap_or(&Decimal::ZERO);
        let new_symbol_exposure = symbol_exposure + order_value;
        let symbol_concentration = new_symbol_exposure / new_total_exposure;

        if symbol_concentration > limits.max_symbol_concentration {
            risk_factors.push(RiskFactor {
                factor_type: "SYMBOL_CONCENTRATION".to_string(),
                severity: RiskSeverity::High,
                value: symbol_concentration * Decimal::from(100),
                threshold: limits.max_symbol_concentration * Decimal::from(100),
                description: "Symbol concentration risk".to_string(),
            });
        }

        Ok(())
    }

    /// 检查市场风险
    async fn check_market_risk(
        &self,
        order: &Order,
        risk_factors: &mut Vec<RiskFactor>,
    ) -> TradingResult<()> {
        let monitor = self.risk_monitor.read().await;
        let limits = self.system_limits.read().await;

        // 检查波动率
        if let Some(&volatility) = monitor.current_volatility.get(&order.symbol) {
            if volatility > limits.volatility_threshold {
                risk_factors.push(RiskFactor {
                    factor_type: "HIGH_VOLATILITY".to_string(),
                    severity: RiskSeverity::Medium,
                    value: volatility * Decimal::from(100),
                    threshold: limits.volatility_threshold * Decimal::from(100),
                    description: "High market volatility".to_string(),
                });
            }
        }

        Ok(())
    }

    /// 评估整体风险等级
    fn assess_overall_risk(&self, risk_factors: &[RiskFactor]) -> RiskLevel {
        let max_severity = risk_factors.iter()
            .map(|f| &f.severity)
            .max()
            .unwrap_or(&RiskSeverity::Low);

        match max_severity {
            RiskSeverity::Low => RiskLevel::Low,
            RiskSeverity::Medium => RiskLevel::Medium,
            RiskSeverity::High => RiskLevel::High,
            RiskSeverity::Critical => RiskLevel::Extreme,
        }
    }

    /// 生成风险建议
    fn generate_recommendations(&self, risk_factors: &[RiskFactor], recommendations: &mut Vec<String>) {
        for factor in risk_factors {
            match factor.factor_type.as_str() {
                "ORDER_VALUE_WARNING" => {
                    recommendations.push("Consider reducing order size".to_string());
                }
                "HIGH_VOLATILITY" => {
                    recommendations.push("Market volatility is high, consider using limit orders".to_string());
                }
                "SYMBOL_CONCENTRATION" => {
                    recommendations.push("Consider diversifying across different symbols".to_string());
                }
                _ => {}
            }
        }
    }

    /// 计算最大允许交易量
    async fn calculate_max_allowed_size(
        &self,
        order: &Order,
        config: &UserRiskConfig,
    ) -> TradingResult<Decimal> {
        // 基于多个因素计算最大允许交易量
        let mut max_size = order.quantity;

        // 基于订单价值限制
        if let Some(price) = order.price {
            let max_size_by_value = config.max_order_value / price;
            max_size = max_size.min(max_size_by_value);
        }

        // 基于仓位限制
        // TODO: 考虑当前仓位

        // 基于保证金限制
        // TODO: 考虑可用保证金

        Ok(max_size)
    }

    /// 实时风险监控更新
    pub async fn update_risk_monitor(&self, positions: &[Position]) -> TradingResult<()> {
        let mut monitor = self.risk_monitor.write().await;
        
        // 重置计数器
        monitor.total_exposure = Decimal::ZERO;
        monitor.symbol_exposures.clear();
        monitor.user_exposures.clear();

        // 计算当前敞口
        for position in positions {
            let position_value = position.get_position_value();
            
            monitor.total_exposure += position_value;
            
            *monitor.symbol_exposures.entry(position.symbol.clone()).or_insert(Decimal::ZERO) += position_value;
            *monitor.user_exposures.entry(position.user_id).or_insert(Decimal::ZERO) += position_value;
        }

        monitor.last_update = chrono::Utc::now();

        // 检查是否触发风险事件
        self.check_risk_thresholds(&monitor).await?;

        Ok(())
    }

    /// 检查风险阈值
    async fn check_risk_thresholds(&self, monitor: &RiskMonitor) -> TradingResult<()> {
        let limits = self.system_limits.read().await;

        // 检查总敞口
        if monitor.total_exposure > limits.max_total_exposure {
            self.trigger_risk_event(RiskEvent {
                event_id: Uuid::new_v4(),
                event_type: RiskEventType::SystemExposureExceeded,
                user_id: None,
                symbol: None,
                severity: RiskSeverity::Critical,
                message: "System exposure limit exceeded".to_string(),
                data: serde_json::json!({
                    "current_exposure": monitor.total_exposure,
                    "limit": limits.max_total_exposure
                }),
                timestamp: chrono::Utc::now(),
                resolved: false,
            }).await;
        }

        // 检查交易对集中度
        for (symbol, exposure) in &monitor.symbol_exposures {
            let concentration = exposure / monitor.total_exposure;
            if concentration > limits.max_symbol_concentration {
                self.trigger_risk_event(RiskEvent {
                    event_id: Uuid::new_v4(),
                    event_type: RiskEventType::ConcentrationRisk,
                    user_id: None,
                    symbol: Some(symbol.clone()),
                    severity: RiskSeverity::High,
                    message: format!("Symbol concentration risk for {}", symbol),
                    data: serde_json::json!({
                        "symbol": symbol,
                        "concentration": concentration,
                        "limit": limits.max_symbol_concentration
                    }),
                    timestamp: chrono::Utc::now(),
                    resolved: false,
                }).await;
            }
        }

        Ok(())
    }

    /// 触发风险事件
    async fn trigger_risk_event(&self, event: RiskEvent) {
        tracing::warn!("Risk event triggered: {:?}", event);
        
        let mut events = self.risk_events.write().await;
        events.push(event.clone());

        // 保留最近1000个事件
        if events.len() > 1000 {
            let len = events.len();
            events.drain(0..len - 1000);
        }

        // TODO: 发送告警通知
        self.send_risk_alert(&event).await;
    }

    /// 发送风险告警
    async fn send_risk_alert(&self, event: &RiskEvent) {
        // TODO: 实现告警通知（邮件、短信、Webhook等）
        tracing::error!("RISK ALERT: {} - {}", event.event_type, event.message);
    }

    /// 获取风险事件历史
    pub async fn get_risk_events(&self, limit: Option<usize>) -> Vec<RiskEvent> {
        let events = self.risk_events.read().await;
        let limit = limit.unwrap_or(100);
        
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取风险监控状态
    pub async fn get_risk_monitor_status(&self) -> RiskMonitor {
        self.risk_monitor.read().await.clone()
    }

    /// 强制平仓检查
    pub async fn check_liquidation_requirements(&self, positions: &[Position]) -> Vec<Uuid> {
        let mut liquidation_list = Vec::new();

        for position in positions {
            if let Some(config) = self.get_user_risk_config(position.user_id).await {
                // 检查是否达到强平阈值
                if position.margin_ratio <= config.liquidation_threshold {
                    liquidation_list.push(position.id);
                    
                    // 触发强平事件
                    let event = RiskEvent {
                        event_id: Uuid::new_v4(),
                        event_type: RiskEventType::Liquidation,
                        user_id: Some(position.user_id),
                        symbol: Some(position.symbol.clone()),
                        severity: RiskSeverity::Critical,
                        message: "Position requires liquidation".to_string(),
                        data: serde_json::json!({
                            "position_id": position.id,
                            "margin_ratio": position.margin_ratio,
                            "threshold": config.liquidation_threshold
                        }),
                        timestamp: chrono::Utc::now(),
                        resolved: false,
                    };
                    
                    self.trigger_risk_event(event).await;
                }
            }
        }

        liquidation_list
    }
}