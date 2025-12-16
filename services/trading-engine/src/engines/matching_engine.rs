use anyhow::Result;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{Order, OrderType, Side, Symbol, TradingError, TradingResult};

/// 高性能订单撮合引擎
/// 使用价格-时间优先算法，支持微秒级撮合
#[derive(Debug)]
pub struct MatchingEngine {
    symbol: Symbol,
    /// 买单订单簿 (价格从高到低)
    bid_orders: Arc<RwLock<BTreeMap<Decimal, VecDeque<Order>>>>,
    /// 卖单订单簿 (价格从低到高)  
    ask_orders: Arc<RwLock<BTreeMap<Decimal, VecDeque<Order>>>>,
    /// 最新成交价
    last_price: Arc<RwLock<Option<Decimal>>>,
    /// 成交统计
    stats: Arc<RwLock<MatchingStats>>,
}

#[derive(Debug, Clone)]
pub struct MatchingStats {
    pub total_volume: Decimal,
    pub total_trades: u64,
    pub avg_trade_size: Decimal,
    pub price_high_24h: Option<Decimal>,
    pub price_low_24h: Option<Decimal>,
    pub volume_24h: Decimal,
}

#[derive(Debug, Clone)]
pub struct TradeExecution {
    pub trade_id: Uuid,
    pub maker_order_id: Uuid,
    pub taker_order_id: Uuid,
    pub symbol: Symbol,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: Side,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
}

#[derive(Debug, Clone)]
pub struct OrderBookSnapshot {
    pub symbol: Symbol,
    pub bids: Vec<(Decimal, Decimal)>, // (price, quantity)
    pub asks: Vec<(Decimal, Decimal)>,
    pub last_price: Option<Decimal>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MatchingEngine {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            bid_orders: Arc::new(RwLock::new(BTreeMap::new())),
            ask_orders: Arc::new(RwLock::new(BTreeMap::new())),
            last_price: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(MatchingStats {
                total_volume: Decimal::ZERO,
                total_trades: 0,
                avg_trade_size: Decimal::ZERO,
                price_high_24h: None,
                price_low_24h: None,
                volume_24h: Decimal::ZERO,
            })),
        }
    }

    /// 处理新订单 - 核心撮合逻辑
    pub async fn process_order(&self, mut order: Order) -> TradingResult<Vec<TradeExecution>> {
        let mut trades = Vec::new();

        match order.order_type {
            OrderType::Market => {
                trades = self.process_market_order(&mut order).await?;
            }
            OrderType::Limit => {
                trades = self.process_limit_order(&mut order).await?;
            }
            _ => {
                return Err(TradingError::InvalidOrder(
                    "Unsupported order type for matching".to_string(),
                ));
            }
        }

        // 更新统计信息
        self.update_stats(&trades).await;

        Ok(trades)
    }

    /// 处理市价单
    async fn process_market_order(&self, order: &mut Order) -> TradingResult<Vec<TradeExecution>> {
        let mut trades = Vec::new();
        let mut remaining_qty = order.quantity;

        match order.side {
            Side::Buy => {
                // 买入市价单，从最低卖价开始撮合
                let mut ask_orders = self.ask_orders.write().await;
                let mut prices_to_remove = Vec::new();

                for (&price, orders_at_price) in ask_orders.iter_mut() {
                    if remaining_qty <= Decimal::ZERO {
                        break;
                    }

                    while let Some(mut maker_order) = orders_at_price.pop_front() {
                        if remaining_qty <= Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                            break;
                        }

                        let trade_qty = remaining_qty.min(maker_order.remaining_quantity);
                        
                        // 创建成交记录
                        let trade = TradeExecution {
                            trade_id: Uuid::new_v4(),
                            maker_order_id: maker_order.id,
                            taker_order_id: order.id,
                            symbol: self.symbol.clone(),
                            price,
                            quantity: trade_qty,
                            side: Side::Buy,
                            timestamp: chrono::Utc::now(),
                            maker_fee: self.calculate_maker_fee(trade_qty, price),
                            taker_fee: self.calculate_taker_fee(trade_qty, price),
                        };

                        trades.push(trade);

                        // 更新订单状态
                        maker_order.filled_quantity += trade_qty;
                        maker_order.remaining_quantity -= trade_qty;
                        remaining_qty -= trade_qty;

                        // 如果maker订单完全成交，不放回队列
                        if maker_order.remaining_quantity > Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                        }
                    }

                    if orders_at_price.is_empty() {
                        prices_to_remove.push(price);
                    }
                }

                // 清理空的价格层级
                for price in prices_to_remove {
                    ask_orders.remove(&price);
                }
            }
            Side::Sell => {
                // 卖出市价单，从最高买价开始撮合
                let mut bid_orders = self.bid_orders.write().await;
                let mut prices_to_remove = Vec::new();

                for (&price, orders_at_price) in bid_orders.iter_mut().rev() {
                    if remaining_qty <= Decimal::ZERO {
                        break;
                    }

                    while let Some(mut maker_order) = orders_at_price.pop_front() {
                        if remaining_qty <= Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                            break;
                        }

                        let trade_qty = remaining_qty.min(maker_order.remaining_quantity);
                        
                        let trade = TradeExecution {
                            trade_id: Uuid::new_v4(),
                            maker_order_id: maker_order.id,
                            taker_order_id: order.id,
                            symbol: self.symbol.clone(),
                            price,
                            quantity: trade_qty,
                            side: Side::Sell,
                            timestamp: chrono::Utc::now(),
                            maker_fee: self.calculate_maker_fee(trade_qty, price),
                            taker_fee: self.calculate_taker_fee(trade_qty, price),
                        };

                        trades.push(trade);

                        maker_order.filled_quantity += trade_qty;
                        maker_order.remaining_quantity -= trade_qty;
                        remaining_qty -= trade_qty;

                        if maker_order.remaining_quantity > Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                        }
                    }

                    if orders_at_price.is_empty() {
                        prices_to_remove.push(price);
                    }
                }

                for price in prices_to_remove {
                    bid_orders.remove(&price);
                }
            }
        }

        // 更新最新成交价
        if let Some(last_trade) = trades.last() {
            *self.last_price.write().await = Some(last_trade.price);
        }

        Ok(trades)
    }

    /// 处理限价单
    async fn process_limit_order(&self, order: &mut Order) -> TradingResult<Vec<TradeExecution>> {
        let order_price = order.price.ok_or_else(|| {
            TradingError::InvalidOrder("Limit order must have price".to_string())
        })?;

        let mut trades = Vec::new();
        let mut remaining_qty = order.quantity;

        match order.side {
            Side::Buy => {
                // 买入限价单，尝试与卖单撮合
                let mut ask_orders = self.ask_orders.write().await;
                let mut prices_to_remove = Vec::new();

                for (&ask_price, orders_at_price) in ask_orders.iter_mut() {
                    if ask_price > order_price || remaining_qty <= Decimal::ZERO {
                        break;
                    }

                    while let Some(mut maker_order) = orders_at_price.pop_front() {
                        if remaining_qty <= Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                            break;
                        }

                        let trade_qty = remaining_qty.min(maker_order.remaining_quantity);
                        
                        let trade = TradeExecution {
                            trade_id: Uuid::new_v4(),
                            maker_order_id: maker_order.id,
                            taker_order_id: order.id,
                            symbol: self.symbol.clone(),
                            price: ask_price, // 使用maker价格
                            quantity: trade_qty,
                            side: Side::Buy,
                            timestamp: chrono::Utc::now(),
                            maker_fee: self.calculate_maker_fee(trade_qty, ask_price),
                            taker_fee: self.calculate_taker_fee(trade_qty, ask_price),
                        };

                        trades.push(trade);

                        maker_order.filled_quantity += trade_qty;
                        maker_order.remaining_quantity -= trade_qty;
                        remaining_qty -= trade_qty;

                        if maker_order.remaining_quantity > Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                        }
                    }

                    if orders_at_price.is_empty() {
                        prices_to_remove.push(ask_price);
                    }
                }

                for price in prices_to_remove {
                    ask_orders.remove(&price);
                }

                // 如果还有剩余数量，加入买单订单簿
                if remaining_qty > Decimal::ZERO {
                    drop(ask_orders); // 释放写锁
                    let mut bid_orders = self.bid_orders.write().await;
                    
                    order.remaining_quantity = remaining_qty;
                    order.filled_quantity = order.quantity - remaining_qty;
                    
                    bid_orders
                        .entry(order_price)
                        .or_insert_with(VecDeque::new)
                        .push_back(order.clone());
                }
            }
            Side::Sell => {
                // 卖出限价单，尝试与买单撮合
                let mut bid_orders = self.bid_orders.write().await;
                let mut prices_to_remove = Vec::new();

                for (&bid_price, orders_at_price) in bid_orders.iter_mut().rev() {
                    if bid_price < order_price || remaining_qty <= Decimal::ZERO {
                        break;
                    }

                    while let Some(mut maker_order) = orders_at_price.pop_front() {
                        if remaining_qty <= Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                            break;
                        }

                        let trade_qty = remaining_qty.min(maker_order.remaining_quantity);
                        
                        let trade = TradeExecution {
                            trade_id: Uuid::new_v4(),
                            maker_order_id: maker_order.id,
                            taker_order_id: order.id,
                            symbol: self.symbol.clone(),
                            price: bid_price, // 使用maker价格
                            quantity: trade_qty,
                            side: Side::Sell,
                            timestamp: chrono::Utc::now(),
                            maker_fee: self.calculate_maker_fee(trade_qty, bid_price),
                            taker_fee: self.calculate_taker_fee(trade_qty, bid_price),
                        };

                        trades.push(trade);

                        maker_order.filled_quantity += trade_qty;
                        maker_order.remaining_quantity -= trade_qty;
                        remaining_qty -= trade_qty;

                        if maker_order.remaining_quantity > Decimal::ZERO {
                            orders_at_price.push_front(maker_order);
                        }
                    }

                    if orders_at_price.is_empty() {
                        prices_to_remove.push(bid_price);
                    }
                }

                for price in prices_to_remove {
                    bid_orders.remove(&price);
                }

                // 如果还有剩余数量，加入卖单订单簿
                if remaining_qty > Decimal::ZERO {
                    drop(bid_orders);
                    let mut ask_orders = self.ask_orders.write().await;
                    
                    order.remaining_quantity = remaining_qty;
                    order.filled_quantity = order.quantity - remaining_qty;
                    
                    ask_orders
                        .entry(order_price)
                        .or_insert_with(VecDeque::new)
                        .push_back(order.clone());
                }
            }
        }

        // 更新最新成交价
        if let Some(last_trade) = trades.last() {
            *self.last_price.write().await = Some(last_trade.price);
        }

        Ok(trades)
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: Uuid, side: Side, price: Option<Decimal>) -> TradingResult<bool> {
        match side {
            Side::Buy => {
                let mut bid_orders = self.bid_orders.write().await;
                if let Some(price) = price {
                    if let Some(orders_at_price) = bid_orders.get_mut(&price) {
                        if let Some(pos) = orders_at_price.iter().position(|o| o.id == order_id) {
                            orders_at_price.remove(pos);
                            if orders_at_price.is_empty() {
                                bid_orders.remove(&price);
                            }
                            return Ok(true);
                        }
                    }
                }
            }
            Side::Sell => {
                let mut ask_orders = self.ask_orders.write().await;
                if let Some(price) = price {
                    if let Some(orders_at_price) = ask_orders.get_mut(&price) {
                        if let Some(pos) = orders_at_price.iter().position(|o| o.id == order_id) {
                            orders_at_price.remove(pos);
                            if orders_at_price.is_empty() {
                                ask_orders.remove(&price);
                            }
                            return Ok(true);
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    /// 获取订单簿快照
    pub async fn get_order_book(&self, depth: usize) -> OrderBookSnapshot {
        let bid_orders = self.bid_orders.read().await;
        let ask_orders = self.ask_orders.read().await;
        let last_price = *self.last_price.read().await;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // 获取买单 (价格从高到低)
        for (&price, orders) in bid_orders.iter().rev().take(depth) {
            let total_qty: Decimal = orders.iter().map(|o| o.remaining_quantity).sum();
            if total_qty > Decimal::ZERO {
                bids.push((price, total_qty));
            }
        }

        // 获取卖单 (价格从低到高)
        for (&price, orders) in ask_orders.iter().take(depth) {
            let total_qty: Decimal = orders.iter().map(|o| o.remaining_quantity).sum();
            if total_qty > Decimal::ZERO {
                asks.push((price, total_qty));
            }
        }

        OrderBookSnapshot {
            symbol: self.symbol.clone(),
            bids,
            asks,
            last_price,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 获取最佳买卖价
    pub async fn get_best_bid_ask(&self) -> (Option<Decimal>, Option<Decimal>) {
        let bid_orders = self.bid_orders.read().await;
        let ask_orders = self.ask_orders.read().await;

        let best_bid = bid_orders.keys().next_back().copied();
        let best_ask = ask_orders.keys().next().copied();

        (best_bid, best_ask)
    }

    /// 计算maker手续费
    fn calculate_maker_fee(&self, quantity: Decimal, price: Decimal) -> Decimal {
        let notional = quantity * price;
        notional * Decimal::new(1, 4) // 0.01% maker fee
    }

    /// 计算taker手续费
    fn calculate_taker_fee(&self, quantity: Decimal, price: Decimal) -> Decimal {
        let notional = quantity * price;
        notional * Decimal::new(2, 4) // 0.02% taker fee
    }

    /// 更新统计信息
    async fn update_stats(&self, trades: &[TradeExecution]) {
        if trades.is_empty() {
            return;
        }

        let mut stats = self.stats.write().await;
        
        for trade in trades {
            stats.total_trades += 1;
            stats.total_volume += trade.quantity;
            stats.volume_24h += trade.quantity;

            // 更新24小时高低价
            if stats.price_high_24h.is_none() || Some(trade.price) > stats.price_high_24h {
                stats.price_high_24h = Some(trade.price);
            }
            if stats.price_low_24h.is_none() || Some(trade.price) < stats.price_low_24h {
                stats.price_low_24h = Some(trade.price);
            }
        }

        // 更新平均成交量
        if stats.total_trades > 0 {
            stats.avg_trade_size = stats.total_volume / Decimal::from(stats.total_trades);
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> MatchingStats {
        self.stats.read().await.clone()
    }

    /// 清理过期订单
    pub async fn cleanup_expired_orders(&self) -> TradingResult<Vec<Uuid>> {
        let now = chrono::Utc::now();
        let mut expired_orders = Vec::new();

        // 清理买单
        let mut bid_orders = self.bid_orders.write().await;
        let mut prices_to_remove = Vec::new();
        
        for (&price, orders_at_price) in bid_orders.iter_mut() {
            orders_at_price.retain(|order| {
                if let Some(expires_at) = order.expires_at {
                    if now > expires_at {
                        expired_orders.push(order.id);
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
            
            if orders_at_price.is_empty() {
                prices_to_remove.push(price);
            }
        }
        
        for price in prices_to_remove {
            bid_orders.remove(&price);
        }

        // 清理卖单
        let mut ask_orders = self.ask_orders.write().await;
        let mut prices_to_remove = Vec::new();
        
        for (&price, orders_at_price) in ask_orders.iter_mut() {
            orders_at_price.retain(|order| {
                if let Some(expires_at) = order.expires_at {
                    if now > expires_at {
                        expired_orders.push(order.id);
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
            
            if orders_at_price.is_empty() {
                prices_to_remove.push(price);
            }
        }
        
        for price in prices_to_remove {
            ask_orders.remove(&price);
        }

        Ok(expired_orders)
    }
}