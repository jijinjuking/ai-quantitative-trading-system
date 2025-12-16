use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 交易配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub enabled: bool,
    pub max_orders_per_user: u32,
    pub max_order_size: Decimal,
    pub min_order_size: Decimal,
    pub order_timeout: Duration,
    pub supported_symbols: Vec<String>,
    pub supported_order_types: Vec<OrderTypeConfig>,
    pub fee_config: FeeConfig,
    pub market_hours: MarketHoursConfig,
}

/// 订单类型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTypeConfig {
    pub order_type: String,
    pub enabled: bool,
    pub min_size: Option<Decimal>,
    pub max_size: Option<Decimal>,
    pub requires_price: bool,
    pub requires_stop_price: bool,
}

/// 手续费配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub withdrawal_fee: Decimal,
    pub minimum_fee: Decimal,
    pub fee_currency: String,
}

/// 市场时间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHoursConfig {
    pub enabled: bool,
    pub timezone: String,
    pub trading_hours: Vec<TradingHour>,
    pub holidays: Vec<String>,
}

/// 交易时间段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingHour {
    pub day_of_week: u8,    // 0=Sunday, 1=Monday, ..., 6=Saturday
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
}

impl TradingConfig {
    /// 验证交易配置
    pub fn validate(&self) -> Result<()> {
        if self.max_orders_per_user == 0 {
            return Err(anyhow::anyhow!("Max orders per user cannot be 0"));
        }

        if self.max_order_size <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Max order size must be positive"));
        }

        if self.min_order_size <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Min order size must be positive"));
        }

        if self.min_order_size >= self.max_order_size {
            return Err(anyhow::anyhow!(
                "Min order size must be less than max order size"
            ));
        }

        // 验证手续费配置
        self.fee_config.validate()?;

        // 验证订单类型配置
        for order_type in &self.supported_order_types {
            order_type.validate()?;
        }

        Ok(())
    }

    /// 检查订单大小是否有效
    pub fn is_valid_order_size(&self, size: Decimal) -> bool {
        size >= self.min_order_size && size <= self.max_order_size
    }

    /// 检查交易对是否支持
    pub fn is_symbol_supported(&self, symbol: &str) -> bool {
        self.supported_symbols.is_empty() || self.supported_symbols.contains(&symbol.to_string())
    }

    /// 检查订单类型是否支持
    pub fn is_order_type_supported(&self, order_type: &str) -> bool {
        self.supported_order_types
            .iter()
            .any(|config| config.order_type == order_type && config.enabled)
    }

    /// 获取订单类型配置
    pub fn get_order_type_config(&self, order_type: &str) -> Option<&OrderTypeConfig> {
        self.supported_order_types
            .iter()
            .find(|config| config.order_type == order_type)
    }
}

impl FeeConfig {
    /// 验证手续费配置
    pub fn validate(&self) -> Result<()> {
        if self.maker_fee < Decimal::ZERO {
            return Err(anyhow::anyhow!("Maker fee cannot be negative"));
        }

        if self.taker_fee < Decimal::ZERO {
            return Err(anyhow::anyhow!("Taker fee cannot be negative"));
        }

        if self.withdrawal_fee < Decimal::ZERO {
            return Err(anyhow::anyhow!("Withdrawal fee cannot be negative"));
        }

        if self.minimum_fee < Decimal::ZERO {
            return Err(anyhow::anyhow!("Minimum fee cannot be negative"));
        }

        if self.fee_currency.is_empty() {
            return Err(anyhow::anyhow!("Fee currency is required"));
        }

        Ok(())
    }

    /// 计算交易手续费
    pub fn calculate_fee(&self, amount: Decimal, is_maker: bool) -> Decimal {
        let fee_rate = if is_maker {
            self.maker_fee
        } else {
            self.taker_fee
        };
        let calculated_fee = amount * fee_rate;
        calculated_fee.max(self.minimum_fee)
    }
}

impl OrderTypeConfig {
    /// 验证订单类型配置
    pub fn validate(&self) -> Result<()> {
        if self.order_type.is_empty() {
            return Err(anyhow::anyhow!("Order type name is required"));
        }

        if let (Some(min), Some(max)) = (self.min_size, self.max_size) {
            if min >= max {
                return Err(anyhow::anyhow!(
                    "Min size must be less than max size for order type {}",
                    self.order_type
                ));
            }
        }

        Ok(())
    }

    /// 检查订单大小是否在范围内
    pub fn is_size_valid(&self, size: Decimal) -> bool {
        if let Some(min) = self.min_size {
            if size < min {
                return false;
            }
        }

        if let Some(max) = self.max_size {
            if size > max {
                return false;
            }
        }

        true
    }
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_orders_per_user: 100,
            max_order_size: Decimal::from(1_000_000),
            min_order_size: Decimal::new(1, 3),        // 0.001
            order_timeout: Duration::from_secs(86400), // 24 hours
            supported_symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "ADAUSDT".to_string(),
                "DOTUSDT".to_string(),
                "LINKUSDT".to_string(),
            ],
            supported_order_types: vec![
                OrderTypeConfig {
                    order_type: "MARKET".to_string(),
                    enabled: true,
                    min_size: None,
                    max_size: None,
                    requires_price: false,
                    requires_stop_price: false,
                },
                OrderTypeConfig {
                    order_type: "LIMIT".to_string(),
                    enabled: true,
                    min_size: None,
                    max_size: None,
                    requires_price: true,
                    requires_stop_price: false,
                },
                OrderTypeConfig {
                    order_type: "STOP_LOSS".to_string(),
                    enabled: true,
                    min_size: None,
                    max_size: None,
                    requires_price: false,
                    requires_stop_price: true,
                },
                OrderTypeConfig {
                    order_type: "TAKE_PROFIT".to_string(),
                    enabled: true,
                    min_size: None,
                    max_size: None,
                    requires_price: false,
                    requires_stop_price: true,
                },
            ],
            fee_config: FeeConfig::default(),
            market_hours: MarketHoursConfig::default(),
        }
    }
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            maker_fee: Decimal::new(1, 3),      // 0.1%
            taker_fee: Decimal::new(15, 4),     // 0.15%
            withdrawal_fee: Decimal::new(5, 3), // 0.5%
            minimum_fee: Decimal::new(1, 6),    // 0.000001
            fee_currency: "USDT".to_string(),
        }
    }
}

impl Default for MarketHoursConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 24/7 trading by default
            timezone: "UTC".to_string(),
            trading_hours: vec![
                TradingHour {
                    day_of_week: 1, // Monday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
                TradingHour {
                    day_of_week: 2, // Tuesday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
                TradingHour {
                    day_of_week: 3, // Wednesday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
                TradingHour {
                    day_of_week: 4, // Thursday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
                TradingHour {
                    day_of_week: 5, // Friday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
                TradingHour {
                    day_of_week: 6, // Saturday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
                TradingHour {
                    day_of_week: 0, // Sunday
                    start_time: "00:00".to_string(),
                    end_time: "23:59".to_string(),
                },
            ],
            holidays: vec![],
        }
    }
}
