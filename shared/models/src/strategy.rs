use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::common::{Exchange, Interval};
use crate::trading::{OrderSide, SignalType};

/// 策略定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub strategy_type: StrategyType,
    pub status: StrategyStatus,
    pub parameters: StrategyParameters,
    pub risk_settings: RiskSettings,
    pub performance: StrategyPerformance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
}

/// 策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    TechnicalAnalysis,
    Arbitrage,
    MarketMaking,
    Momentum,
    MeanReversion,
    MachineLearning,
    Custom,
}

/// 策略状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyStatus {
    Draft,
    Testing,
    Active,
    Paused,
    Stopped,
    Error,
}

/// 策略参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    pub symbols: Vec<String>,
    pub exchanges: Vec<Exchange>,
    pub timeframes: Vec<Interval>,
    pub indicators: HashMap<String, IndicatorConfig>,
    pub entry_conditions: Vec<Condition>,
    pub exit_conditions: Vec<Condition>,
    pub position_sizing: PositionSizing,
    pub custom_params: HashMap<String, serde_json::Value>,
}

/// 指标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorConfig {
    pub indicator_type: IndicatorType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub period: u32,
    pub source: PriceSource,
}

/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorType {
    SMA,      // Simple Moving Average
    EMA,      // Exponential Moving Average
    RSI,      // Relative Strength Index
    MACD,     // Moving Average Convergence Divergence
    BB,       // Bollinger Bands
    Stoch,    // Stochastic Oscillator
    ATR,      // Average True Range
    ADX,      // Average Directional Index
    CCI,      // Commodity Channel Index
    Williams, // Williams %R
    Custom,
}

/// 价格源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceSource {
    Open,
    High,
    Low,
    Close,
    Volume,
    HL2,   // (High + Low) / 2
    HLC3,  // (High + Low + Close) / 3
    OHLC4, // (Open + High + Low + Close) / 4
}

/// 条件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub description: String,
    pub left_operand: Operand,
    pub operator: ComparisonOperator,
    pub right_operand: Operand,
    pub logical_operator: Option<LogicalOperator>,
}

/// 操作数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operand {
    Indicator { name: String, shift: u32 },
    Price { source: PriceSource, shift: u32 },
    Value(Decimal),
    Variable(String),
}

/// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
    CrossAbove,
    CrossBelow,
}

/// 逻辑操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// 仓位大小计算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizing {
    pub method: PositionSizingMethod,
    pub base_amount: Decimal,
    pub risk_percentage: Option<Decimal>,
    pub max_position_size: Option<Decimal>,
    pub leverage: Option<Decimal>,
}

/// 仓位大小计算方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    Fixed,           // 固定金额
    Percentage,      // 账户百分比
    RiskBased,       // 基于风险
    VolatilityBased, // 基于波动率
    KellyFormula,    // 凯利公式
}

/// 风险设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    pub max_drawdown: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub max_daily_loss: Option<Decimal>,
    pub max_positions: Option<u32>,
    pub max_correlation: Option<Decimal>,
    pub trailing_stop: Option<TrailingStop>,
}

/// 追踪止损
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStop {
    pub distance: Decimal,
    pub distance_type: TrailingStopType,
}

/// 追踪止损类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrailingStopType {
    Fixed,      // 固定点数
    Percentage, // 百分比
    ATR,        // ATR倍数
}

/// 策略性能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub total_return: Decimal,
    pub annualized_return: Decimal,
    pub max_drawdown: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub updated_at: DateTime<Utc>,
}

/// 策略信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub symbol: String,
    pub exchange: Exchange,
    pub side: OrderSide,
    pub signal_type: SignalType,
    pub strength: Decimal,
    pub price: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub confidence: Decimal,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_capital: Decimal,
    pub commission: Decimal,
    pub slippage: Decimal,
    pub benchmark: Option<String>,
    pub data_frequency: Interval,
    pub created_at: DateTime<Utc>,
}

/// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: Uuid,
    pub config_id: Uuid,
    pub status: BacktestStatus,
    pub performance: StrategyPerformance,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<BacktestTrade>,
    pub metrics: BacktestMetrics,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// 回测状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BacktestStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 权益曲线点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: Decimal,
    pub drawdown: Decimal,
    pub benchmark: Option<Decimal>,
}

/// 回测交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub quantity: Decimal,
    pub pnl: Decimal,
    pub commission: Decimal,
    pub duration: i64, // 持仓时间（秒）
}

/// 回测指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: Decimal,
    pub annualized_return: Decimal,
    pub volatility: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_duration: i64,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub beta: Option<Decimal>,
    pub alpha: Option<Decimal>,
    pub information_ratio: Option<Decimal>,
}

/// 策略优化参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationParameter {
    pub name: String,
    pub min_value: Decimal,
    pub max_value: Decimal,
    pub step: Decimal,
    pub current_value: Decimal,
}

/// 优化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub parameters: HashMap<String, Decimal>,
    pub performance: StrategyPerformance,
    pub rank: u32,
    pub created_at: DateTime<Utc>,
}

/// 实时策略状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRuntime {
    pub strategy_id: Uuid,
    pub status: StrategyStatus,
    pub current_positions: HashMap<String, Decimal>,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub total_trades: u64,
    pub last_signal: Option<DateTime<Utc>>,
    pub last_trade: Option<DateTime<Utc>>,
    pub error_count: u32,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}
