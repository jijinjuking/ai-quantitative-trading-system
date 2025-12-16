use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: &str, message: &str) {
        self.is_valid = false;
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        });
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// 验证器
pub struct Validator;

impl Validator {
    /// 验证必填字段
    pub fn required(value: &Option<String>, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value.is_none() || value.as_ref().unwrap().trim().is_empty() {
            result.add_error(field, "This field is required");
        }
        result
    }

    /// 验证字符串长度
    pub fn length(value: &str, min: usize, max: usize, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let len = value.len();
        if len < min {
            result.add_error(field, &format!("Minimum length is {}", min));
        }
        if len > max {
            result.add_error(field, &format!("Maximum length is {}", max));
        }
        result
    }

    /// 验证最小长度
    pub fn min_length(value: &str, min: usize, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value.len() < min {
            result.add_error(field, &format!("Minimum length is {}", min));
        }
        result
    }

    /// 验证最大长度
    pub fn max_length(value: &str, max: usize, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value.len() > max {
            result.add_error(field, &format!("Maximum length is {}", max));
        }
        result
    }

    /// 验证邮箱格式
    pub fn email(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(value) {
            result.add_error(field, "Invalid email format");
        }
        result
    }

    /// 验证手机号格式
    pub fn phone(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
        if !phone_regex.is_match(value) {
            result.add_error(field, "Invalid phone number format");
        }
        result
    }

    /// 验证用户名格式
    pub fn username(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let username_regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
        if !username_regex.is_match(value) {
            result.add_error(field, "Username must be 3-20 characters and contain only letters, numbers, and underscores");
        }
        result
    }

    /// 验证密码强度
    pub fn password(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();

        if value.len() < 8 {
            result.add_error(field, "Password must be at least 8 characters long");
        }

        if !value.chars().any(|c| c.is_lowercase()) {
            result.add_error(field, "Password must contain at least one lowercase letter");
        }

        if !value.chars().any(|c| c.is_uppercase()) {
            result.add_error(field, "Password must contain at least one uppercase letter");
        }

        if !value.chars().any(|c| c.is_numeric()) {
            result.add_error(field, "Password must contain at least one number");
        }

        if !value
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        {
            result.add_error(
                field,
                "Password must contain at least one special character",
            );
        }

        result
    }

    /// 验证数值范围
    pub fn number_range(
        value: Decimal,
        min: Decimal,
        max: Decimal,
        field: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value < min {
            result.add_error(field, &format!("Value must be at least {}", min));
        }
        if value > max {
            result.add_error(field, &format!("Value must be at most {}", max));
        }
        result
    }

    /// 验证最小值
    pub fn min_value(value: Decimal, min: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value < min {
            result.add_error(field, &format!("Value must be at least {}", min));
        }
        result
    }

    /// 验证最大值
    pub fn max_value(value: Decimal, max: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value > max {
            result.add_error(field, &format!("Value must be at most {}", max));
        }
        result
    }

    /// 验证正数
    pub fn positive(value: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value <= Decimal::ZERO {
            result.add_error(field, "Value must be positive");
        }
        result
    }

    /// 验证非负数
    pub fn non_negative(value: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if value < Decimal::ZERO {
            result.add_error(field, "Value must be non-negative");
        }
        result
    }

    /// 验证URL格式
    pub fn url(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        if !url_regex.is_match(value) {
            result.add_error(field, "Invalid URL format");
        }
        result
    }

    /// 验证UUID格式
    pub fn uuid(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if uuid::Uuid::parse_str(value).is_err() {
            result.add_error(field, "Invalid UUID format");
        }
        result
    }

    /// 验证日期格式
    pub fn date(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
            result.add_error(field, "Invalid date format (YYYY-MM-DD)");
        }
        result
    }

    /// 验证时间格式
    pub fn datetime(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        if chrono::DateTime::parse_from_rfc3339(value).is_err() {
            result.add_error(field, "Invalid datetime format (RFC3339)");
        }
        result
    }

    /// 验证枚举值
    pub fn enum_value<T: AsRef<str>>(
        value: &str,
        allowed_values: &[T],
        field: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        if !allowed_values.iter().any(|v| v.as_ref() == value) {
            let allowed: Vec<&str> = allowed_values.iter().map(|v| v.as_ref()).collect();
            result.add_error(
                field,
                &format!("Value must be one of: {}", allowed.join(", ")),
            );
        }
        result
    }

    /// 验证数组长度
    pub fn array_length<T>(value: &[T], min: usize, max: usize, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let len = value.len();
        if len < min {
            result.add_error(field, &format!("Array must have at least {} items", min));
        }
        if len > max {
            result.add_error(field, &format!("Array must have at most {} items", max));
        }
        result
    }

    /// 验证唯一性
    pub fn unique<T: PartialEq>(value: &[T], field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        for (i, item) in value.iter().enumerate() {
            if value.iter().skip(i + 1).any(|other| item == other) {
                result.add_error(field, "Array must contain unique values");
                break;
            }
        }
        result
    }

    /// 自定义验证
    pub fn custom<F>(value: &str, validator: F, field: &str, message: &str) -> ValidationResult
    where
        F: Fn(&str) -> bool,
    {
        let mut result = ValidationResult::new();
        if !validator(value) {
            result.add_error(field, message);
        }
        result
    }
}

/// 交易相关验证器
pub struct TradingValidator;

impl TradingValidator {
    /// 验证交易对格式
    pub fn symbol(value: &str, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let symbol_regex = Regex::new(r"^[A-Z]{2,10}[A-Z]{2,10}$").unwrap();
        if !symbol_regex.is_match(value) {
            result.add_error(field, "Invalid symbol format (e.g., BTCUSDT)");
        }
        result
    }

    /// 验证订单数量
    pub fn order_quantity(value: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.merge(Validator::positive(value, field));

        // 检查精度（最多8位小数）
        if value.scale() > 8 {
            result.add_error(field, "Quantity precision cannot exceed 8 decimal places");
        }

        result
    }

    /// 验证订单价格
    pub fn order_price(value: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.merge(Validator::positive(value, field));

        // 检查精度（最多8位小数）
        if value.scale() > 8 {
            result.add_error(field, "Price precision cannot exceed 8 decimal places");
        }

        result
    }

    /// 验证杠杆倍数
    pub fn leverage(value: Decimal, field: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.merge(Validator::number_range(
            value,
            Decimal::ONE,
            Decimal::from(125),
            field,
        ));
        result
    }

    /// 验证止损价格
    pub fn stop_loss(
        value: Decimal,
        entry_price: Decimal,
        side: &str,
        field: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.merge(Validator::positive(value, field));

        match side.to_uppercase().as_str() {
            "BUY" => {
                if value >= entry_price {
                    result.add_error(field, "Stop loss must be below entry price for buy orders");
                }
            }
            "SELL" => {
                if value <= entry_price {
                    result.add_error(field, "Stop loss must be above entry price for sell orders");
                }
            }
            _ => {
                result.add_error("side", "Invalid order side");
            }
        }

        result
    }

    /// 验证止盈价格
    pub fn take_profit(
        value: Decimal,
        entry_price: Decimal,
        side: &str,
        field: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.merge(Validator::positive(value, field));

        match side.to_uppercase().as_str() {
            "BUY" => {
                if value <= entry_price {
                    result.add_error(
                        field,
                        "Take profit must be above entry price for buy orders",
                    );
                }
            }
            "SELL" => {
                if value >= entry_price {
                    result.add_error(
                        field,
                        "Take profit must be below entry price for sell orders",
                    );
                }
            }
            _ => {
                result.add_error("side", "Invalid order side");
            }
        }

        result
    }
}

/// 验证器构建器
pub struct ValidatorBuilder {
    result: ValidationResult,
}

impl ValidatorBuilder {
    pub fn new() -> Self {
        Self {
            result: ValidationResult::new(),
        }
    }

    pub fn required(mut self, value: &Option<String>, field: &str) -> Self {
        self.result.merge(Validator::required(value, field));
        self
    }

    pub fn length(mut self, value: &str, min: usize, max: usize, field: &str) -> Self {
        self.result.merge(Validator::length(value, min, max, field));
        self
    }

    pub fn email(mut self, value: &str, field: &str) -> Self {
        self.result.merge(Validator::email(value, field));
        self
    }

    pub fn password(mut self, value: &str, field: &str) -> Self {
        self.result.merge(Validator::password(value, field));
        self
    }

    pub fn positive(mut self, value: Decimal, field: &str) -> Self {
        self.result.merge(Validator::positive(value, field));
        self
    }

    pub fn custom<F>(mut self, value: &str, validator: F, field: &str, message: &str) -> Self
    where
        F: Fn(&str) -> bool,
    {
        self.result
            .merge(Validator::custom(value, validator, field, message));
        self
    }

    pub fn build(self) -> ValidationResult {
        self.result
    }
}

impl Default for ValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 批量验证器
pub struct BatchValidator {
    validators: HashMap<String, Box<dyn Fn() -> ValidationResult>>,
}

impl BatchValidator {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    pub fn add<F>(mut self, field: &str, validator: F) -> Self
    where
        F: Fn() -> ValidationResult + 'static,
    {
        self.validators
            .insert(field.to_string(), Box::new(validator));
        self
    }

    pub fn validate(self) -> ValidationResult {
        let mut result = ValidationResult::new();
        for (_, validator) in self.validators {
            result.merge(validator());
        }
        result
    }
}

impl Default for BatchValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validation() {
        let result = Validator::required(&None, "field");
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        let result = Validator::required(&Some("value".to_string()), "field");
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_email_validation() {
        let result = Validator::email("invalid-email", "email");
        assert!(!result.is_valid);

        let result = Validator::email("test@example.com", "email");
        assert!(result.is_valid);
    }

    #[test]
    fn test_password_validation() {
        let result = Validator::password("weak", "password");
        assert!(!result.is_valid);
        assert!(result.errors.len() > 1);

        let result = Validator::password("StrongPass123!", "password");
        assert!(result.is_valid);
    }

    #[test]
    fn test_number_validation() {
        let result = Validator::positive(Decimal::from(-1), "amount");
        assert!(!result.is_valid);

        let result = Validator::positive(Decimal::from(10), "amount");
        assert!(result.is_valid);
    }

    #[test]
    fn test_trading_validation() {
        let result = TradingValidator::symbol("BTCUSDT", "symbol");
        assert!(result.is_valid);

        let result = TradingValidator::symbol("invalid", "symbol");
        assert!(!result.is_valid);

        let result = TradingValidator::order_quantity(Decimal::from(10), "quantity");
        assert!(result.is_valid);

        let result = TradingValidator::order_quantity(Decimal::from(-1), "quantity");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validator_builder() {
        let result = ValidatorBuilder::new()
            .required(&Some("test@example.com".to_string()), "email")
            .email("test@example.com", "email")
            .length("test@example.com", 5, 50, "email")
            .build();

        assert!(result.is_valid);
    }

    #[test]
    fn test_batch_validator() {
        let result = BatchValidator::new()
            .add("email", || Validator::email("test@example.com", "email"))
            .add("password", || {
                Validator::password("StrongPass123!", "password")
            })
            .validate();

        assert!(result.is_valid);
    }
}
