use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

/// 时间工具
pub struct TimeUtils;

impl TimeUtils {
    /// 获取当前UTC时间
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }

    /// 获取当前时间戳（秒）
    pub fn timestamp() -> i64 {
        Utc::now().timestamp()
    }

    /// 获取当前时间戳（毫秒）
    pub fn timestamp_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    /// DateTime转换为时间戳（毫秒）
    pub fn datetime_to_timestamp_millis(datetime: &DateTime<Utc>) -> i64 {
        datetime.timestamp_millis()
    }

    /// DateTime转换为时间戳（秒）
    pub fn datetime_to_timestamp(datetime: &DateTime<Utc>) -> i64 {
        datetime.timestamp()
    }

    /// 获取当前时间戳（微秒）
    pub fn timestamp_micros() -> i64 {
        Utc::now().timestamp_micros()
    }

    /// 从时间戳创建DateTime（秒）
    pub fn from_timestamp(timestamp: i64) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(timestamp, 0).single()
    }

    /// 从时间戳创建DateTime（毫秒）
    pub fn from_timestamp_millis(timestamp: i64) -> Option<DateTime<Utc>> {
        Utc.timestamp_millis_opt(timestamp).single()
    }

    /// 从时间戳创建DateTime（微秒）
    pub fn from_timestamp_micros(timestamp: i64) -> Option<DateTime<Utc>> {
        Utc.timestamp_micros(timestamp).single()
    }

    /// 格式化时间为ISO 8601字符串
    pub fn to_iso_string(datetime: &DateTime<Utc>) -> String {
        datetime.to_rfc3339()
    }

    /// 从ISO 8601字符串解析时间
    pub fn from_iso_string(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
    }

    /// 格式化时间为自定义格式
    pub fn format(datetime: &DateTime<Utc>, format: &str) -> String {
        datetime.format(format).to_string()
    }

    /// 解析自定义格式的时间字符串
    pub fn parse(s: &str, format: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
        let naive = NaiveDateTime::parse_from_str(s, format)?;
        Ok(Utc.from_utc_datetime(&naive))
    }

    /// 获取今天的开始时间（00:00:00）
    pub fn start_of_day(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        datetime
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    /// 获取今天的结束时间（23:59:59）
    pub fn end_of_day(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        datetime
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
    }

    /// 获取本周的开始时间（周一00:00:00）
    pub fn start_of_week(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        let weekday = datetime.weekday().num_days_from_monday();
        let start_of_day = Self::start_of_day(datetime);
        start_of_day - Duration::days(weekday as i64)
    }

    /// 获取本月的开始时间（1号00:00:00）
    pub fn start_of_month(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        datetime
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    /// 获取本年的开始时间（1月1号00:00:00）
    pub fn start_of_year(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        datetime
            .date_naive()
            .with_month(1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    /// 添加时间
    pub fn add_duration(datetime: &DateTime<Utc>, duration: Duration) -> DateTime<Utc> {
        *datetime + duration
    }

    /// 减去时间
    pub fn sub_duration(datetime: &DateTime<Utc>, duration: Duration) -> DateTime<Utc> {
        *datetime - duration
    }

    /// 计算两个时间的差值
    pub fn duration_between(start: &DateTime<Utc>, end: &DateTime<Utc>) -> Duration {
        *end - *start
    }

    /// 检查时间是否在范围内
    pub fn is_between(
        datetime: &DateTime<Utc>,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> bool {
        datetime >= start && datetime <= end
    }

    /// 获取时间范围
    pub fn get_time_range(period: TimePeriod) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Self::now();
        match period {
            TimePeriod::LastHour => (now - Duration::hours(1), now),
            TimePeriod::Last24Hours => (now - Duration::hours(24), now),
            TimePeriod::Last7Days => (now - Duration::days(7), now),
            TimePeriod::Last30Days => (now - Duration::days(30), now),
            TimePeriod::LastMonth => {
                let start = Self::start_of_month(&(now - Duration::days(30)));
                let end = Self::start_of_month(&now) - Duration::seconds(1);
                (start, end)
            }
            TimePeriod::LastYear => {
                let start = Self::start_of_year(&(now - Duration::days(365)));
                let end = Self::start_of_year(&now) - Duration::seconds(1);
                (start, end)
            }
            TimePeriod::Today => (Self::start_of_day(&now), Self::end_of_day(&now)),
            TimePeriod::ThisWeek => (Self::start_of_week(&now), now),
            TimePeriod::ThisMonth => (Self::start_of_month(&now), now),
            TimePeriod::ThisYear => (Self::start_of_year(&now), now),
        }
    }

    /// 将时间舍入到指定间隔
    pub fn round_to_interval(datetime: &DateTime<Utc>, interval: TimeInterval) -> DateTime<Utc> {
        let timestamp = datetime.timestamp();
        let interval_seconds = interval.to_seconds();
        let rounded_timestamp = (timestamp / interval_seconds) * interval_seconds;
        Self::from_timestamp(rounded_timestamp).unwrap_or(*datetime)
    }

    /// 生成时间序列
    pub fn generate_time_series(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: TimeInterval,
    ) -> Vec<DateTime<Utc>> {
        let mut series = Vec::new();
        let mut current = start;
        let interval_duration = Duration::seconds(interval.to_seconds());

        while current <= end {
            series.push(current);
            current = current + interval_duration;
        }

        series
    }

    /// 检查是否为工作日
    pub fn is_weekday(datetime: &DateTime<Utc>) -> bool {
        let weekday = datetime.weekday();
        !matches!(weekday, Weekday::Sat | Weekday::Sun)
    }

    /// 检查是否为周末
    pub fn is_weekend(datetime: &DateTime<Utc>) -> bool {
        !Self::is_weekday(datetime)
    }

    /// 获取下一个工作日
    pub fn next_weekday(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        let mut next = *datetime + Duration::days(1);
        while !Self::is_weekday(&next) {
            next = next + Duration::days(1);
        }
        next
    }

    /// 获取上一个工作日
    pub fn prev_weekday(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        let mut prev = *datetime - Duration::days(1);
        while !Self::is_weekday(&prev) {
            prev = prev - Duration::days(1);
        }
        prev
    }
}

/// 时间周期枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePeriod {
    LastHour,
    Last24Hours,
    Last7Days,
    Last30Days,
    LastMonth,
    LastYear,
    Today,
    ThisWeek,
    ThisMonth,
    ThisYear,
}

/// 时间间隔枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInterval {
    Second,
    Minute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    Hour,
    FourHours,
    Day,
    Week,
    Month,
}

impl TimeInterval {
    pub fn to_seconds(&self) -> i64 {
        match self {
            TimeInterval::Second => 1,
            TimeInterval::Minute => 60,
            TimeInterval::FiveMinutes => 300,
            TimeInterval::FifteenMinutes => 900,
            TimeInterval::ThirtyMinutes => 1800,
            TimeInterval::Hour => 3600,
            TimeInterval::FourHours => 14400,
            TimeInterval::Day => 86400,
            TimeInterval::Week => 604800,
            TimeInterval::Month => 2592000, // 30 days
        }
    }
}

/// 时区工具
pub struct TimezoneUtils;

impl TimezoneUtils {
    /// 转换到指定时区
    pub fn to_timezone<Tz: TimeZone>(datetime: &DateTime<Utc>, timezone: &Tz) -> DateTime<Tz> {
        datetime.with_timezone(timezone)
    }

    /// 从指定时区转换到UTC
    pub fn to_utc<Tz: TimeZone>(datetime: &DateTime<Tz>) -> DateTime<Utc> {
        datetime.with_timezone(&Utc)
    }

    /// 获取常用时区的时间
    pub fn get_common_timezones(datetime: &DateTime<Utc>) -> CommonTimezones {
        use chrono_tz::{Asia::Shanghai, Europe::London, US::Eastern, US::Pacific};

        CommonTimezones {
            utc: *datetime,
            new_york: datetime.with_timezone(&Eastern),
            london: datetime.with_timezone(&London),
            shanghai: datetime.with_timezone(&Shanghai),
            los_angeles: datetime.with_timezone(&Pacific),
        }
    }
}

/// 常用时区时间
#[derive(Debug, Clone)]
pub struct CommonTimezones {
    pub utc: DateTime<Utc>,
    pub new_york: DateTime<chrono_tz::Tz>,
    pub london: DateTime<chrono_tz::Tz>,
    pub shanghai: DateTime<chrono_tz::Tz>,
    pub los_angeles: DateTime<chrono_tz::Tz>,
}

/// 交易时间工具
pub struct TradingTimeUtils;

impl TradingTimeUtils {
    /// 检查是否为交易时间（美股）
    pub fn is_us_market_open(datetime: &DateTime<Utc>) -> bool {
        use chrono_tz::US::Eastern;
        let eastern_time = datetime.with_timezone(&Eastern);

        // 检查是否为工作日
        if !TimeUtils::is_weekday(&datetime) {
            return false;
        }

        // 美股交易时间：9:30 - 16:00 ET
        let hour = eastern_time.hour();
        let minute = eastern_time.minute();

        (hour > 9 || (hour == 9 && minute >= 30)) && hour < 16
    }

    /// 检查是否为交易时间（亚洲市场）
    pub fn is_asian_market_open(datetime: &DateTime<Utc>) -> bool {
        use chrono_tz::Asia::Shanghai;
        let shanghai_time = datetime.with_timezone(&Shanghai);

        // 检查是否为工作日
        if !TimeUtils::is_weekday(&datetime) {
            return false;
        }

        // 亚洲市场交易时间：9:00 - 15:00 CST
        let hour = shanghai_time.hour();
        hour >= 9 && hour < 15
    }

    /// 检查是否为交易时间（欧洲市场）
    pub fn is_european_market_open(datetime: &DateTime<Utc>) -> bool {
        use chrono_tz::Europe::London;
        let london_time = datetime.with_timezone(&London);

        // 检查是否为工作日
        if !TimeUtils::is_weekday(&datetime) {
            return false;
        }

        // 欧洲市场交易时间：8:00 - 16:30 GMT
        let hour = london_time.hour();
        let minute = london_time.minute();

        hour >= 8 && (hour < 16 || (hour == 16 && minute <= 30))
    }

    /// 获取下一个交易日开盘时间（美股）
    pub fn next_us_market_open(datetime: &DateTime<Utc>) -> DateTime<Utc> {
        use chrono_tz::US::Eastern;

        let next_day = TimeUtils::next_weekday(datetime);
        let eastern_time = next_day.with_timezone(&Eastern);

        // 设置为9:30 ET
        let market_open = eastern_time
            .date_naive()
            .and_hms_opt(9, 30, 0)
            .unwrap()
            .and_local_timezone(Eastern)
            .single()
            .unwrap()
            .with_timezone(&Utc);

        market_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_utils() {
        let now = TimeUtils::now();
        let timestamp = TimeUtils::timestamp();
        let from_timestamp = TimeUtils::from_timestamp(timestamp).unwrap();

        // 允许1秒的误差
        assert!((now.timestamp() - from_timestamp.timestamp()).abs() <= 1);
    }

    #[test]
    fn test_time_formatting() {
        let datetime = TimeUtils::from_timestamp(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
        let iso_string = TimeUtils::to_iso_string(&datetime);
        let parsed = TimeUtils::from_iso_string(&iso_string).unwrap();

        assert_eq!(datetime, parsed);
    }

    #[test]
    fn test_time_ranges() {
        let now = TimeUtils::now();
        let start_of_day = TimeUtils::start_of_day(&now);
        let end_of_day = TimeUtils::end_of_day(&now);

        assert!(start_of_day <= now);
        assert!(now <= end_of_day);
        assert!(TimeUtils::is_between(&now, &start_of_day, &end_of_day));
    }

    #[test]
    fn test_time_intervals() {
        assert_eq!(TimeInterval::Minute.to_seconds(), 60);
        assert_eq!(TimeInterval::Hour.to_seconds(), 3600);
        assert_eq!(TimeInterval::Day.to_seconds(), 86400);
    }

    #[test]
    fn test_weekday_detection() {
        // 2022-01-03 是周一
        let monday = TimeUtils::from_timestamp(1641168000).unwrap();
        assert!(TimeUtils::is_weekday(&monday));
        assert!(!TimeUtils::is_weekend(&monday));

        // 2022-01-01 是周六
        let saturday = TimeUtils::from_timestamp(1640995200).unwrap();
        assert!(!TimeUtils::is_weekday(&saturday));
        assert!(TimeUtils::is_weekend(&saturday));
    }
}
