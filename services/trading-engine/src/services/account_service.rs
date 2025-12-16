use anyhow::Result;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{TradingError, TradingResult},
    services::PositionService,
    storage::AccountStore,
};
use shared_models::AccountType;

/// 账户服务
#[derive(Clone)]
pub struct AccountService {
    account_store: Arc<AccountStore>,
    position_service: Arc<PositionService>,
}

#[derive(Debug, serde::Serialize)]
pub struct AccountInfo {
    pub user_id: Uuid,
    pub account_type: AccountType,
    pub total_balance: Decimal,
    pub available_balance: Decimal,
    pub frozen_balance: Decimal,
    pub total_equity: Decimal,
    pub unrealized_pnl: Decimal,
    pub margin_used: Decimal,
    pub margin_available: Decimal,
    pub margin_ratio: Decimal,
}

#[derive(Debug, serde::Serialize)]
pub struct BalanceInfo {
    pub currency: String,
    pub total: Decimal,
    pub available: Decimal,
    pub frozen: Decimal,
}

#[derive(Debug, serde::Serialize)]
pub struct MarginInfo {
    pub total_margin: Decimal,
    pub used_margin: Decimal,
    pub available_margin: Decimal,
    pub margin_ratio: Decimal,
    pub maintenance_margin: Decimal,
    pub liquidation_threshold: Decimal,
}

#[derive(Debug, serde::Serialize)]
pub struct PnLSummary {
    pub total_unrealized_pnl: Decimal,
    pub total_realized_pnl: Decimal,
    pub daily_pnl: Decimal,
    pub weekly_pnl: Decimal,
    pub monthly_pnl: Decimal,
    pub total_pnl: Decimal,
    pub roi: Decimal,
}

impl AccountService {
    pub fn new(
        account_store: Arc<AccountStore>,
        position_service: Arc<PositionService>,
    ) -> Self {
        Self {
            account_store,
            position_service,
        }
    }

    /// 获取账户信息
    pub async fn get_account(
        &self,
        user_id: Uuid,
        account_type: Option<AccountType>,
    ) -> TradingResult<AccountInfo> {
        // TODO: 从数据库获取账户信息
        // 这里返回模拟数据
        let account_type = account_type.unwrap_or(AccountType::Spot);
        
        let total_balance = Decimal::from(100000);
        let frozen_balance = Decimal::from(5000);
        let available_balance = total_balance - frozen_balance;
        
        let unrealized_pnl = self.position_service.calculate_total_pnl(user_id).await
            .unwrap_or(Decimal::ZERO);
        
        let margin_used = self.position_service.calculate_total_margin(user_id).await
            .unwrap_or(Decimal::ZERO);
        
        let total_equity = total_balance + unrealized_pnl;
        let margin_available = total_equity - margin_used;
        let margin_ratio = if margin_used > Decimal::ZERO {
            margin_available / margin_used
        } else {
            Decimal::ZERO
        };

        Ok(AccountInfo {
            user_id,
            account_type,
            total_balance,
            available_balance,
            frozen_balance,
            total_equity,
            unrealized_pnl,
            margin_used,
            margin_available,
            margin_ratio,
        })
    }

    /// 获取余额信息
    pub async fn get_balance(
        &self,
        user_id: Uuid,
        account_type: Option<AccountType>,
    ) -> TradingResult<Vec<BalanceInfo>> {
        // TODO: 从数据库获取真实余额
        // 这里返回模拟数据
        Ok(vec![
            BalanceInfo {
                currency: "USDT".to_string(),
                total: Decimal::from(100000),
                available: Decimal::from(95000),
                frozen: Decimal::from(5000),
            },
            BalanceInfo {
                currency: "BTC".to_string(),
                total: Decimal::new(2, 0),
                available: Decimal::new(15, 1),
                frozen: Decimal::new(5, 1),
            },
        ])
    }

    /// 获取保证金信息
    pub async fn get_margin_info(&self, user_id: Uuid) -> TradingResult<MarginInfo> {
        let total_margin = Decimal::from(50000);
        let used_margin = self.position_service.calculate_total_margin(user_id).await
            .unwrap_or(Decimal::from(20000));
        let available_margin = total_margin - used_margin;
        let margin_ratio = if used_margin > Decimal::ZERO {
            available_margin / used_margin
        } else {
            Decimal::ZERO
        };
        let maintenance_margin = used_margin * Decimal::new(5, 2); // 5%
        let liquidation_threshold = Decimal::new(3, 2); // 3%

        Ok(MarginInfo {
            total_margin,
            used_margin,
            available_margin,
            margin_ratio,
            maintenance_margin,
            liquidation_threshold,
        })
    }

    /// 获取盈亏统计
    pub async fn get_pnl_summary(&self, user_id: Uuid) -> TradingResult<PnLSummary> {
        let total_unrealized_pnl = self.position_service.calculate_total_pnl(user_id).await
            .unwrap_or(Decimal::ZERO);
        
        // TODO: 从数据库获取已实现盈亏和历史数据
        let total_realized_pnl = Decimal::from(5000);
        let daily_pnl = Decimal::from(500);
        let weekly_pnl = Decimal::from(2000);
        let monthly_pnl = Decimal::from(8000);
        let total_pnl = total_unrealized_pnl + total_realized_pnl;
        
        let initial_capital = Decimal::from(100000);
        let roi = if initial_capital > Decimal::ZERO {
            total_pnl / initial_capital * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        Ok(PnLSummary {
            total_unrealized_pnl,
            total_realized_pnl,
            daily_pnl,
            weekly_pnl,
            monthly_pnl,
            total_pnl,
            roi,
        })
    }
}