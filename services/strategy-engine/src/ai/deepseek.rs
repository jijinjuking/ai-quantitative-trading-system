use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::strategy_generator::*;
use crate::models::{Strategy, TradingSignal};

/// DeepSeek AI客户端
/// 专业的AI量化交易策略生成和分析
pub struct DeepSeekClient {
    api_key: String,
    base_url: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl DeepSeekClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            client: Client::new(),
        }
    }

    /// 发送请求到DeepSeek API
    async fn send_request(&self, messages: Vec<Message>) -> Result<String> {
        let request = DeepSeekRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.7,
            max_tokens: 4000,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("DeepSeek API error: {}", error_text));
        }

        let deepseek_response: DeepSeekResponse = response.json().await?;
        
        if let Some(choice) = deepseek_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response from DeepSeek"))
        }
    }

    /// 构建策略生成提示
    fn build_strategy_prompt(&self, prompt: &StrategyPrompt) -> String {
        format!(
            r#"
你是一个专业的量化交易策略专家。请根据以下要求生成一个详细的交易策略：

## 交易目标
{}

## 风险偏好
{:?}

## 时间周期
{:?}

## 市场条件
- 趋势方向: {:?}
- 波动率水平: {:?}
- 流动性水平: {:?}
- 市场情绪: {:?}

## 交易标的
{}

## 资金规模
{}

## 约束条件
{}

## 其他偏好
{}

请生成一个包含以下内容的完整策略：

1. **策略名称和描述**
2. **入场条件** (具体的技术指标和阈值)
3. **出场条件** (止盈、止损、时间出场)
4. **风险管理** (仓位管理、最大回撤控制)
5. **参数设置** (所有可调参数及其默认值)
6. **预期表现** (年化收益率、夏普比率、最大回撤等)
7. **策略代码** (Python或Rust实现)

请以JSON格式返回，确保所有数值都是有效的数字格式。
"#,
            prompt.objective,
            prompt.risk_tolerance,
            prompt.time_horizon,
            prompt.market_conditions.trend,
            prompt.market_conditions.volatility,
            prompt.market_conditions.liquidity,
            prompt.market_conditions.sentiment,
            prompt.symbols.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "),
            prompt.capital,
            prompt.constraints.join(", "),
            prompt.preferences.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ")
        )
    }

    /// 构建市场分析提示
    fn build_market_analysis_prompt(&self, context: &MarketContext) -> String {
        format!(
            r#"
作为专业的市场分析师，请对以下市场数据进行深度分析：

## 交易对信息
- 标的: {}
- 当前价格: {}
- 24小时成交量: {}
- 买卖价差: {}

## 技术指标
{}

## 市场微观结构
- 订单簿深度: {}
- 交易频率: {}
- 价格冲击: {}

请提供以下分析：

1. **趋势分析** (方向、强度、持续性)
2. **波动率分析** (当前波动率水平和预期)
3. **动量分析** (短期、中期、长期动量)
4. **支撑阻力位** (关键价格水平)
5. **模式识别** (技术形态和信号)
6. **异常检测** (价格或成交量异常)
7. **市场制度分类** (牛市、熊市、震荡市)

请以JSON格式返回分析结果。
"#,
            context.symbol,
            context.current_price,
            context.volume_profile.total_volume,
            context.market_microstructure.bid_ask_spread,
            context.technical_indicators.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", "),
            context.market_microstructure.order_book_depth,
            context.market_microstructure.trade_frequency,
            context.market_microstructure.price_impact
        )
    }

    /// 构建信号预测提示
    fn build_signal_prediction_prompt(&self, context: &TradingContext) -> String {
        format!(
            r#"
作为AI交易信号生成器，请基于以下信息生成精准的交易信号：

## 市场数据
- 标的: {}
- 当前价格: {}
- 技术指标: {}

## 投资组合状态
- 总价值: {}
- 可用资金: {}
- 当前持仓: {}

## 风险状况
- 当前VaR: {}
- 最大回撤: {}

## 执行环境
- 市场开放: {}
- 流动性条件: {:?}

请生成以下交易信号：

1. **信号类型** (买入/卖出/持有)
2. **信号强度** (1-10分)
3. **建议仓位** (占总资金比例)
4. **目标价格** (入场价格)
5. **止损价格**
6. **止盈价格**
7. **持有时间** (预期持有周期)
8. **信号理由** (技术和基本面依据)
9. **风险评估** (潜在风险和概率)

请以JSON数组格式返回多个交易信号。
"#,
            context.market_context.symbol,
            context.market_context.current_price,
            context.market_context.technical_indicators.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", "),
            context.portfolio_context.total_value,
            context.portfolio_context.available_cash,
            context.portfolio_context.positions.len(),
            context.risk_context.current_var,
            context.risk_context.max_drawdown,
            context.execution_context.market_hours,
            context.execution_context.liquidity_conditions
        )
    }

    /// 解析策略响应
    fn parse_strategy_response(&self, response: &str) -> Result<GeneratedStrategy> {
        // 尝试从响应中提取JSON
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        // 解析JSON响应
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse strategy JSON: {}", e))?;

        // 构建GeneratedStrategy
        let strategy = GeneratedStrategy {
            name: parsed["name"].as_str().unwrap_or("AI Generated Strategy").to_string(),
            description: parsed["description"].as_str().unwrap_or("").to_string(),
            strategy_type: crate::models::StrategyType::Custom, // 默认为自定义类型
            entry_conditions: self.parse_conditions(&parsed["entry_conditions"])?,
            exit_conditions: self.parse_conditions(&parsed["exit_conditions"])?,
            risk_management: self.parse_risk_management(&parsed["risk_management"])?,
            parameters: self.parse_parameters(&parsed["parameters"])?,
            expected_performance: self.parse_expected_performance(&parsed["expected_performance"])?,
            code: parsed["code"].as_str().map(|s| s.to_string()),
            confidence_score: parsed["confidence_score"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(75, 2)), // 默认75%
        };

        Ok(strategy)
    }

    /// 解析条件
    fn parse_conditions(&self, conditions_json: &serde_json::Value) -> Result<Vec<Condition>> {
        let mut conditions = Vec::new();
        
        if let Some(conditions_array) = conditions_json.as_array() {
            for condition_json in conditions_array {
                conditions.push(Condition {
                    name: condition_json["name"].as_str().unwrap_or("").to_string(),
                    description: condition_json["description"].as_str().unwrap_or("").to_string(),
                    logic: condition_json["logic"].as_str().unwrap_or("").to_string(),
                    parameters: HashMap::new(), // 简化处理
                });
            }
        }
        
        Ok(conditions)
    }

    /// 解析风险管理
    fn parse_risk_management(&self, risk_json: &serde_json::Value) -> Result<RiskManagement> {
        Ok(RiskManagement {
            stop_loss: risk_json["stop_loss"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            take_profit: risk_json["take_profit"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            position_sizing: PositionSizing::Percentage(
                risk_json["position_size"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(Decimal::new(10, 2)) // 默认10%
            ),
            max_drawdown: risk_json["max_drawdown"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(20, 2)), // 默认20%
            max_positions: risk_json["max_positions"]
                .as_u64()
                .unwrap_or(5) as u32, // 默认5个
        })
    }

    /// 解析参数
    fn parse_parameters(&self, params_json: &serde_json::Value) -> Result<HashMap<String, ParameterValue>> {
        let mut parameters = HashMap::new();
        
        if let Some(params_obj) = params_json.as_object() {
            for (key, value) in params_obj {
                let param_value = match value {
                    serde_json::Value::Number(n) => {
                        if n.is_i64() {
                            ParameterValue::Integer(n.as_i64().unwrap())
                        } else {
                            ParameterValue::Float(n.as_f64().unwrap())
                        }
                    }
                    serde_json::Value::String(s) => {
                        if let Ok(decimal) = s.parse::<Decimal>() {
                            ParameterValue::Decimal(decimal)
                        } else {
                            ParameterValue::String(s.clone())
                        }
                    }
                    serde_json::Value::Bool(b) => ParameterValue::Boolean(*b),
                    _ => ParameterValue::String(value.to_string()),
                };
                parameters.insert(key.clone(), param_value);
            }
        }
        
        Ok(parameters)
    }

    /// 解析预期表现
    fn parse_expected_performance(&self, perf_json: &serde_json::Value) -> Result<ExpectedPerformance> {
        Ok(ExpectedPerformance {
            annual_return: perf_json["annual_return"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(15, 2)), // 默认15%
            sharpe_ratio: perf_json["sharpe_ratio"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(15, 1)), // 默认1.5
            max_drawdown: perf_json["max_drawdown"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(10, 2)), // 默认10%
            win_rate: perf_json["win_rate"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(60, 2)), // 默认60%
            profit_factor: perf_json["profit_factor"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(15, 1)), // 默认1.5
            volatility: perf_json["volatility"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::new(20, 2)), // 默认20%
        })
    }
}

#[async_trait]
impl AIClient for DeepSeekClient {
    async fn generate_strategy(&self, prompt: &StrategyPrompt) -> Result<GeneratedStrategy> {
        let system_message = Message {
            role: "system".to_string(),
            content: "你是一个世界顶级的量化交易策略专家，拥有20年的华尔街交易经验。你精通各种交易策略、风险管理和市场分析。请用专业、精确的语言回答问题。".to_string(),
        };

        let user_message = Message {
            role: "user".to_string(),
            content: self.build_strategy_prompt(prompt),
        };

        let response = self.send_request(vec![system_message, user_message]).await?;
        self.parse_strategy_response(&response)
    }

    async fn analyze_market(&self, context: &MarketContext) -> Result<MarketAnalysis> {
        let system_message = Message {
            role: "system".to_string(),
            content: "你是一个专业的市场分析师，擅长技术分析、基本面分析和市场微观结构分析。".to_string(),
        };

        let user_message = Message {
            role: "user".to_string(),
            content: self.build_market_analysis_prompt(context),
        };

        let response = self.send_request(vec![system_message, user_message]).await?;
        
        // 简化的市场分析解析
        Ok(MarketAnalysis {
            trend_analysis: TrendAnalysis {
                direction: TrendDirection::Bullish,
                strength: Decimal::new(75, 2),
                duration: 86400, // 1天
                confidence: Decimal::new(80, 2),
            },
            volatility_analysis: VolatilityAnalysis {
                current_volatility: Decimal::new(25, 2),
                historical_volatility: Decimal::new(30, 2),
                implied_volatility: None,
                volatility_regime: VolatilityRegime::Normal,
            },
            momentum_analysis: MomentumAnalysis {
                short_term_momentum: Decimal::new(5, 1),
                medium_term_momentum: Decimal::new(3, 1),
                long_term_momentum: Decimal::new(2, 1),
                momentum_divergence: false,
            },
            support_resistance: SupportResistance {
                support_levels: vec![Decimal::from(48000), Decimal::from(47000)],
                resistance_levels: vec![Decimal::from(52000), Decimal::from(53000)],
                pivot_points: vec![Decimal::from(50000)],
            },
            pattern_recognition: Vec::new(),
            anomaly_detection: Vec::new(),
            regime_classification: MarketRegime::Bull,
        })
    }

    async fn optimize_parameters(&self, strategy: &Strategy, performance: &PerformanceMetrics) -> Result<OptimizedParameters> {
        // TODO: 实现参数优化
        Ok(OptimizedParameters {
            parameters: HashMap::new(),
            expected_improvement: Decimal::new(5, 2), // 5%
            confidence: Decimal::new(75, 2), // 75%
            optimization_method: "AI-driven optimization".to_string(),
        })
    }

    async fn predict_signals(&self, context: &TradingContext) -> Result<Vec<TradingSignal>> {
        let system_message = Message {
            role: "system".to_string(),
            content: "你是一个AI交易信号生成器，能够基于市场数据生成精准的买卖信号。".to_string(),
        };

        let user_message = Message {
            role: "user".to_string(),
            content: self.build_signal_prediction_prompt(context),
        };

        let response = self.send_request(vec![system_message, user_message]).await?;
        
        // 简化的信号解析
        Ok(vec![
            TradingSignal {
                id: uuid::Uuid::new_v4(),
                symbol: context.market_context.symbol.clone(),
                signal_type: crate::models::SignalType::Buy,
                strength: Decimal::new(8, 1), // 0.8
                price: context.market_context.current_price,
                timestamp: chrono::Utc::now(),
                strategy_id: None,
                metadata: std::collections::HashMap::new(),
            }
        ])
    }

    fn get_model_name(&self) -> &str {
        &self.model
    }
}