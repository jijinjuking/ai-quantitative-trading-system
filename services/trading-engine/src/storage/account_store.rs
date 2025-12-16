use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{TradingError, TradingResult};

/// 账户存储
#[derive(Clone)]
pub struct AccountStore {
    pool: Arc<PgPool>,
}

impl AccountStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // TODO: 实现账户相关的数据库操作
}
