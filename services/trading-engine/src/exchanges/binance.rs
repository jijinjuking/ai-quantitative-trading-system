use anyhow::Result;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::models::{Order, Symbol};
use crate::engines::execution_engine::{MarketData, OrderStatusInfo};

/// 币安交易所连接器
#[derive(Clone)]
pub struct BinanceConnector {
    pub name: String,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
}

impl BinanceConnector {
    pub fn new() -> Self {
        Self {
            name: "Binance".to_string(),
            maker_fee: Decimal::from_f64_retain(0.001).unwrap_or_default(), // 0.1%
            taker_fee: Decimal::from_f64_retain(0.001).unwrap_or_default(), // 0.1%
        }
    }

    pub async fn submit_order(&self, _order: &Order) -> Result<String> {
        // 模拟订单提交
        Ok(uuid::Uuid::new_v4().to_string())
    }

    pub async fn cancel_order(&self, _order_id: &str) -> Result<()> {
        // 模拟订单取消
        Ok(())
    }

    pub async fn get_order_status(&self, _order_id: &str) -> Result<OrderStatusInfo> {
        // 模拟订单状态查询
        Ok(OrderStatusInfo {
            order_id: _order_id.to_string(),
            status: "FILLED".to_string(),
            filled_quantity: Decimal::from(100),
            avg_price: Some(Decimal::from(50000)),
        })
    }

    pub async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>> {
        // 模拟账户余额查询
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), Decimal::from(10000));
        balances.insert("BTC".to_string(), Decimal::from_f64_retain(0.5).unwrap_or_default());
        Ok(balances)
    }

    pub async fn get_market_data(&self, _symbol: &Symbol) -> Result<MarketData> {
        // 模拟市场数据获取
        Ok(MarketData {
            symbol: _symbol.clone(),
            price: Decimal::from(50000),
            volume: Decimal::from(1000),
            timestamp: chrono::Utc::now(),
            bid: Some(Decimal::from(49999)),
            ask: Some(Decimal::from(50001)),
            last: Some(Decimal::from(50000)),
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_fees(&self) -> (Decimal, Decimal) {
        (self.maker_fee, self.taker_fee)
    }
}

impl Default for BinanceConnector {
    fn default() -> Self {
        Self::new()
    }
}