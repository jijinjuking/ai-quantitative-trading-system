use anyhow::Result;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    config::{TradingEngineConfig, execution::RoutingStrategy},
    engines::MatchingEngine,
    models::{Order, OrderType, Side, Symbol, TradingError, TradingResult, OrderStatus},
    exchanges::binance::BinanceConnector,
};

/// 市场数据结构
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: Symbol,
    pub price: Decimal,
    pub volume: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
    pub last: Option<Decimal>,
}

/// 专业级执行引擎
/// 交易所连接器枚举，用于替代dyn trait
#[derive(Clone)]
pub enum ExchangeConnectorEnum {
    Binance(BinanceConnector),
    // 可以添加其他交易所
}

impl ExchangeConnectorEnum {
    pub async fn submit_order(&self, order: &Order) -> Result<String> {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.submit_order(order).await,
        }
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.cancel_order(order_id).await,
        }
    }

    pub async fn get_order_status(&self, order_id: &str) -> Result<OrderStatusInfo> {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.get_order_status(order_id).await,
        }
    }

    pub async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>> {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.get_account_balance().await,
        }
    }

    pub async fn get_market_data(&self, symbol: &Symbol) -> Result<MarketData> {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.get_market_data(symbol).await,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.get_name(),
        }
    }

    pub fn get_fees(&self) -> (Decimal, Decimal) {
        match self {
            ExchangeConnectorEnum::Binance(connector) => connector.get_fees(),
        }
    }
}

/// 支持智能订单路由、算法交易、流动性聚合
#[derive(Clone)]
pub struct ExecutionEngine {
    config: TradingEngineConfig,
    /// 内部撮合引擎池
    matching_engines: Arc<RwLock<HashMap<Symbol, Arc<MatchingEngine>>>>,
    /// 外部交易所连接器
    exchange_connectors: Arc<RwLock<HashMap<String, ExchangeConnectorEnum>>>,
    /// 执行统计
    execution_stats: Arc<RwLock<ExecutionStats>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_orders: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time_ms: f64,
    pub total_volume: Decimal,
    pub total_fees: Decimal,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub order_id: Uuid,
    pub execution_id: Uuid,
    pub status: ExecutionStatus,
    pub filled_quantity: Decimal,
    pub avg_price: Option<Decimal>,
    pub total_fee: Decimal,
    pub execution_time_ms: u64,
    pub venue: String,
    pub trades: Vec<TradeExecution>,
}

#[derive(Debug, Clone)]
pub struct TradeExecution {
    pub trade_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub fee: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Failed,
}

/// 交易所连接器接口（暂时注释掉，使用ExchangeConnectorEnum代替）
/*
#[async_trait::async_trait]
pub trait ExchangeConnector: Send + Sync {
    async fn submit_order(&self, order: &Order) -> Result<String>;
    async fn cancel_order(&self, order_id: &str) -> Result<()>;
    async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus>;
    async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>>;
    async fn get_market_data(&self, symbol: &Symbol) -> Result<MarketData>;
    fn get_name(&self) -> &str;
    fn get_fees(&self) -> (Decimal, Decimal); // (maker_fee, taker_fee)
}
*/

// OrderStatus已在models模块中定义为enum
// RoutingStrategy已在config::execution模块中定义

/// 订单状态信息（用于交易所返回）
#[derive(Debug, Clone)]
pub struct OrderStatusInfo {
    pub order_id: String,
    pub status: String,
    pub filled_quantity: Decimal,
    pub avg_price: Option<Decimal>,
}

impl ExecutionEngine {
    pub async fn new(config: TradingEngineConfig) -> Result<Self> {
        let execution_stats = ExecutionStats {
            total_orders: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            total_volume: Decimal::ZERO,
            total_fees: Decimal::ZERO,
        };

        Ok(Self {
            config,
            matching_engines: Arc::new(RwLock::new(HashMap::new())),
            exchange_connectors: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(execution_stats)),
        })
    }

    /// 注册交易所连接器
    pub async fn register_exchange(&self, connector: ExchangeConnectorEnum) {
        let name = connector.get_name().to_string();
        let mut connectors = self.exchange_connectors.write().await;
        connectors.insert(name.clone(), connector);
        tracing::info!("已注册交易所连接器: {}", name);
    }

    /// 获取或创建撮合引擎
    async fn get_matching_engine(&self, symbol: &Symbol) -> Arc<MatchingEngine> {
        let mut engines = self.matching_engines.write().await;
        engines
            .entry(symbol.clone())
            .or_insert_with(|| Arc::new(MatchingEngine::new(symbol.clone())))
            .clone()
    }

    /// 执行订单 - 智能路由
    pub async fn execute_order(
        &self,
        order: Order,
        strategy: RoutingStrategy,
    ) -> TradingResult<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let execution_id = Uuid::new_v4();

        tracing::info!(
            "Executing order {} with strategy {:?}",
            order.id,
            strategy
        );

        let result = match strategy {
            RoutingStrategy::BestPrice => self.execute_best_price(&order).await,
            RoutingStrategy::LowestFee => self.execute_lowest_fee(&order).await,
            RoutingStrategy::FastestExecution => self.execute_lowest_latency(&order).await,
            RoutingStrategy::SmartRouting => self.execute_best_price(&order).await, // 暂时使用最佳价格
            RoutingStrategy::RoundRobin => self.execute_best_price(&order).await, // 暂时使用最佳价格
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        // 更新统计信息
        self.update_execution_stats(&result, execution_time).await;

        match result {
            Ok(mut exec_result) => {
                exec_result.execution_id = execution_id;
                exec_result.execution_time_ms = execution_time;
                Ok(exec_result)
            }
            Err(e) => {
                tracing::error!("Order execution failed: {}", e);
                Err(e)
            }
        }
    }

    /// 最佳价格执行策略
    async fn execute_best_price(&self, order: &Order) -> TradingResult<ExecutionResult> {
        let venues = self.get_available_venues(&order.symbol).await?;
        let mut best_venue = None;
        let mut best_price = None;

        for venue in venues {
            if let Ok(market_data) = venue.get_market_data(&order.symbol).await {
                let price = match order.side {
                    Side::Buy => market_data.ask,
                    Side::Sell => market_data.bid,
                };

                if let Some(price) = price {
                    if best_price.is_none() || 
                       (order.side == Side::Buy && price < best_price.unwrap()) ||
                       (order.side == Side::Sell && price > best_price.unwrap()) {
                        best_price = Some(price);
                        best_venue = Some(venue);
                    }
                }
            }
        }

        if let Some(venue) = best_venue {
            self.execute_on_venue(order, &venue).await
        } else {
            // 回退到内部撮合引擎
            self.execute_internal(order).await
        }
    }

    /// 最低手续费执行策略
    async fn execute_lowest_fee(&self, order: &Order) -> TradingResult<ExecutionResult> {
        let venues = self.get_available_venues(&order.symbol).await?;
        let mut best_venue = None;
        let mut lowest_fee = None;

        for venue in venues {
            let (maker_fee, taker_fee) = venue.get_fees();
            let fee = if order.order_type == OrderType::Limit { maker_fee } else { taker_fee };

            if lowest_fee.is_none() || fee < lowest_fee.unwrap() {
                lowest_fee = Some(fee);
                best_venue = Some(venue);
            }
        }

        if let Some(venue) = best_venue {
            self.execute_on_venue(order, &venue).await
        } else {
            self.execute_internal(order).await
        }
    }

    /// 最大流动性执行策略
    async fn execute_max_liquidity(&self, order: &Order) -> TradingResult<ExecutionResult> {
        let venues = self.get_available_venues(&order.symbol).await?;
        let mut best_venue = None;
        let mut max_volume = Decimal::ZERO;

        for venue in venues {
            if let Ok(market_data) = venue.get_market_data(&order.symbol).await {
                if market_data.volume > max_volume {
                    max_volume = market_data.volume;
                    best_venue = Some(venue);
                }
            }
        }

        if let Some(venue) = best_venue {
            self.execute_on_venue(order, &venue).await
        } else {
            self.execute_internal(order).await
        }
    }

    /// 最低延迟执行策略
    async fn execute_lowest_latency(&self, order: &Order) -> TradingResult<ExecutionResult> {
        // 优先使用内部撮合引擎（延迟最低）
        self.execute_internal(order).await
    }

    /// 分割执行策略
    async fn execute_split(
        &self,
        order: &Order,
        venues: Vec<(String, Decimal)>,
    ) -> TradingResult<ExecutionResult> {
        let mut total_filled = Decimal::ZERO;
        let mut total_fee = Decimal::ZERO;
        let mut all_trades = Vec::new();
        let mut weighted_price_sum = Decimal::ZERO;
        let mut total_weight = Decimal::ZERO;

        for (venue_name, percentage) in venues {
            let split_quantity = order.quantity * percentage / Decimal::from(100);
            
            if split_quantity <= Decimal::ZERO {
                continue;
            }

            let mut split_order = order.clone();
            split_order.id = Uuid::new_v4();
            split_order.quantity = split_quantity;
            split_order.remaining_quantity = split_quantity;

            // 尝试在指定交易所执行
            let connectors = self.exchange_connectors.read().await;
            if let Some(connector) = connectors.get(&venue_name) {
                match self.execute_on_venue(&split_order, connector).await {
                    Ok(result) => {
                        total_filled += result.filled_quantity;
                        total_fee += result.total_fee;
                        all_trades.extend(result.trades);
                        
                        if let Some(avg_price) = result.avg_price {
                            weighted_price_sum += avg_price * result.filled_quantity;
                            total_weight += result.filled_quantity;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to execute on {}: {}", venue_name, e);
                        // 继续执行其他部分
                    }
                }
            }
        }

        let avg_price = if total_weight > Decimal::ZERO {
            Some(weighted_price_sum / total_weight)
        } else {
            None
        };

        let status = if total_filled >= order.quantity {
            ExecutionStatus::Filled
        } else if total_filled > Decimal::ZERO {
            ExecutionStatus::PartiallyFilled
        } else {
            ExecutionStatus::Failed
        };

        Ok(ExecutionResult {
            order_id: order.id,
            execution_id: Uuid::new_v4(),
            status,
            filled_quantity: total_filled,
            avg_price,
            total_fee,
            execution_time_ms: 0, // 将在上层设置
            venue: "SPLIT".to_string(),
            trades: all_trades,
        })
    }

    /// 在指定交易所执行订单
    async fn execute_on_venue(
        &self,
        order: &Order,
        connector: &ExchangeConnectorEnum,
    ) -> TradingResult<ExecutionResult> {
        match connector.submit_order(order).await {
            Ok(exchange_order_id) => {
                // 模拟等待执行完成
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                match connector.get_order_status(&exchange_order_id).await {
                    Ok(status) => {
                        let execution_status = match status.status.as_str() {
                            "FILLED" => ExecutionStatus::Filled,
                            "PARTIALLY_FILLED" => ExecutionStatus::PartiallyFilled,
                            "CANCELLED" => ExecutionStatus::Cancelled,
                            "REJECTED" => ExecutionStatus::Rejected,
                            _ => ExecutionStatus::Pending,
                        };

                        let (maker_fee, taker_fee) = connector.get_fees();
                        let fee_rate = if order.order_type == OrderType::Limit { maker_fee } else { taker_fee };
                        let total_fee = if let Some(avg_price) = status.avg_price {
                            status.filled_quantity * avg_price * fee_rate
                        } else {
                            Decimal::ZERO
                        };

                        let trade = TradeExecution {
                            trade_id: Uuid::new_v4(),
                            price: status.avg_price.unwrap_or(Decimal::ZERO),
                            quantity: status.filled_quantity,
                            timestamp: chrono::Utc::now(),
                            fee: total_fee,
                        };

                        Ok(ExecutionResult {
                            order_id: order.id,
                            execution_id: Uuid::new_v4(),
                            status: execution_status,
                            filled_quantity: status.filled_quantity,
                            avg_price: status.avg_price,
                            total_fee,
                            execution_time_ms: 0,
                            venue: connector.get_name().to_string(),
                            trades: vec![trade],
                        })
                    }
                    Err(e) => Err(TradingError::ExecutionError(format!(
                        "Failed to get order status: {}", e
                    ))),
                }
            }
            Err(e) => Err(TradingError::ExecutionError(format!(
                "Failed to submit order: {}", e
            ))),
        }
    }

    /// 内部撮合引擎执行
    async fn execute_internal(&self, order: &Order) -> TradingResult<ExecutionResult> {
        let matching_engine = self.get_matching_engine(&order.symbol).await;
        
        match matching_engine.process_order(order.clone()).await {
            Ok(trades) => {
                let total_filled: Decimal = trades.iter().map(|t| t.quantity).sum();
                let total_fee: Decimal = trades.iter().map(|t| t.taker_fee).sum();
                
                let avg_price = if total_filled > Decimal::ZERO {
                    let weighted_sum: Decimal = trades.iter()
                        .map(|t| t.price * t.quantity)
                        .sum();
                    Some(weighted_sum / total_filled)
                } else {
                    None
                };

                let status = if total_filled >= order.quantity {
                    ExecutionStatus::Filled
                } else if total_filled > Decimal::ZERO {
                    ExecutionStatus::PartiallyFilled
                } else {
                    ExecutionStatus::Pending
                };

                let trade_executions: Vec<TradeExecution> = trades.into_iter().map(|t| {
                    TradeExecution {
                        trade_id: t.trade_id,
                        price: t.price,
                        quantity: t.quantity,
                        timestamp: t.timestamp,
                        fee: t.taker_fee,
                    }
                }).collect();

                Ok(ExecutionResult {
                    order_id: order.id,
                    execution_id: Uuid::new_v4(),
                    status,
                    filled_quantity: total_filled,
                    avg_price,
                    total_fee,
                    execution_time_ms: 0,
                    venue: "INTERNAL".to_string(),
                    trades: trade_executions,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// 获取可用交易所
    async fn get_available_venues(&self, _symbol: &Symbol) -> TradingResult<Vec<ExchangeConnectorEnum>> {
        let connectors = self.exchange_connectors.read().await;
        Ok(connectors.values().cloned().collect())
    }

    /// 更新执行统计
    async fn update_execution_stats(&self, result: &TradingResult<ExecutionResult>, execution_time: u64) {
        let mut stats = self.execution_stats.write().await;
        stats.total_orders += 1;

        match result {
            Ok(exec_result) => {
                if exec_result.status == ExecutionStatus::Filled || 
                   exec_result.status == ExecutionStatus::PartiallyFilled {
                    stats.successful_executions += 1;
                    stats.total_volume += exec_result.filled_quantity;
                    stats.total_fees += exec_result.total_fee;
                } else {
                    stats.failed_executions += 1;
                }
            }
            Err(_) => {
                stats.failed_executions += 1;
            }
        }

        // 更新平均执行时间
        let total_time = stats.avg_execution_time_ms * (stats.total_orders - 1) as f64 + execution_time as f64;
        stats.avg_execution_time_ms = total_time / stats.total_orders as f64;
    }

    /// 获取执行统计
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: Uuid, venue: Option<String>) -> TradingResult<bool> {
        if let Some(venue_name) = venue {
            let connectors = self.exchange_connectors.read().await;
            if let Some(connector) = connectors.get(&venue_name) {
                match connector.cancel_order(&order_id.to_string()).await {
                    Ok(_) => Ok(true),
                    Err(e) => Err(TradingError::ExecutionError(format!(
                        "Failed to cancel order on {}: {}", venue_name, e
                    ))),
                }
            } else {
                Err(TradingError::ExecutionError(format!(
                    "Unknown venue: {}", venue_name
                )))
            }
        } else {
            // 尝试在所有交易所取消
            let connectors = self.exchange_connectors.read().await;
            let mut cancelled = false;
            
            for connector in connectors.values() {
                if connector.cancel_order(&order_id.to_string()).await.is_ok() {
                    cancelled = true;
                }
            }
            
            Ok(cancelled)
        }
    }

    /// 获取订单簿聚合视图
    pub async fn get_aggregated_order_book(&self, symbol: &Symbol, depth: usize) -> TradingResult<AggregatedOrderBook> {
        let matching_engine = self.get_matching_engine(symbol).await;
        let internal_book = matching_engine.get_order_book(depth).await;
        
        // TODO: 聚合外部交易所的订单簿数据
        
        Ok(AggregatedOrderBook {
            symbol: symbol.clone(),
            bids: internal_book.bids,
            asks: internal_book.asks,
            last_price: internal_book.last_price,
            timestamp: chrono::Utc::now(),
            venues: vec!["INTERNAL".to_string()],
        })
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedOrderBook {
    pub symbol: Symbol,
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
    pub last_price: Option<Decimal>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub venues: Vec<String>,
}