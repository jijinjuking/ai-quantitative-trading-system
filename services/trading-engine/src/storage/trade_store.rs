use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{TradingError, TradingResult};

/// 交易记录存储
#[derive(Clone)]
pub struct TradeStore {
    pool: Arc<PgPool>,
}

impl TradeStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // TODO: 实现交易记录相关的数据库操作
}
