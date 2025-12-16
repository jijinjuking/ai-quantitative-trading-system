use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
use shared_models::common::{Exchange, Interval, DataQuality};

/// K线连续性检测器 - Phase 1 最小实现
pub struct KlineContinuityDetector {
    /// 按 (exchange, symbol, interval) 维度维护最后的 open_time
    last_open_times: Arc<RwLock<HashMap<String, i64>>>,
    /// 内存统计
    stats: Arc<RwLock<ContinuityStats>>,
}

/// 连续性统计信息
#[derive(Debug, Default, Clone)]
pub struct ContinuityStats {
    pub total_checks: u64,
    pub gaps_detected: u64,
    pub last_check_time: Option<i64>,
}

/// 连续性检测结果
#[derive(Debug, Clone)]
pub struct ContinuityCheckResult {
    pub has_gap: bool,
    pub gap_duration_ms: Option<i64>,
    pub expected_next_time: Option<i64>,
    pub actual_time: i64,
    /// 数据质量标记
    pub data_quality: DataQuality,
}

impl KlineContinuityDetector {
    /// 创建新的连续性检测器
    pub fn new() -> Self {
        Self {
            last_open_times: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ContinuityStats::default())),
        }
    }

    /// 检查K线连续性
    /// 参数：exchange, symbol, interval, open_time (毫秒时间戳)
    pub async fn check_continuity(
        &self,
        exchange: Exchange,
        symbol: &str,
        interval: Interval,
        open_time: i64,
    ) -> ContinuityCheckResult {
        // 构建唯一键：exchange:symbol:interval
        let key = format!("{}:{}:{}", 
            exchange.as_str(), 
            symbol, 
            interval.as_str()
        );

        let mut result = ContinuityCheckResult {
            has_gap: false,
            gap_duration_ms: None,
            expected_next_time: None,
            actual_time: open_time,
            data_quality: DataQuality::Normal, // 默认为正常
        };

        // 获取上次的 open_time
        let mut last_times = self.last_open_times.write().await;
        
        if let Some(&last_open_time) = last_times.get(&key) {
            // 计算预期的下一个 open_time (基于interval)
            let interval_ms = self.get_interval_milliseconds(&interval);
            let expected_next_time = last_open_time + interval_ms;
            
            result.expected_next_time = Some(expected_next_time);
            
            // 检查是否有间隙 (根据时间间隔动态调整容忍度)
            let gap_ms = open_time - expected_next_time;
            let tolerance_ms = self.get_tolerance_milliseconds(&interval);
            
            if gap_ms.abs() > tolerance_ms { // 超过容忍范围认为有间隙
                result.has_gap = true;
                result.gap_duration_ms = Some(gap_ms);
                result.data_quality = DataQuality::Suspect; // gap后首次到达的数据标记为可疑
                
                // 输出结构化警告日志
                let gap_type = if gap_ms > 0 { "延迟" } else { "提前" };
                warn!(
                    exchange = %exchange.as_str(),
                    symbol = %symbol,
                    interval = %interval.as_str(),
                    last_open_time = %last_open_time,
                    expected_next_time = %expected_next_time,
                    actual_open_time = %open_time,
                    gap_duration_ms = %gap_ms,
                    gap_type = %gap_type,
                    tolerance_ms = %tolerance_ms,
                    data_quality = %result.data_quality,
                    "K线连续性间隙检测"
                );
                
                // 更新间隙统计
                let mut stats = self.stats.write().await;
                stats.gaps_detected += 1;
            }
        }

        // 更新最后的 open_time
        last_times.insert(key, open_time);
        
        // 更新检查统计
        let mut stats = self.stats.write().await;
        stats.total_checks += 1;
        stats.last_check_time = Some(chrono::Utc::now().timestamp_millis());

        result
    }

    /// 获取时间间隔对应的毫秒数
    fn get_interval_milliseconds(&self, interval: &Interval) -> i64 {
        match interval {
            Interval::OneSecond => 1_000,
            Interval::OneMinute => 60_000,
            Interval::ThreeMinutes => 180_000,
            Interval::FiveMinutes => 300_000,
            Interval::FifteenMinutes => 900_000,
            Interval::ThirtyMinutes => 1_800_000,
            Interval::OneHour => 3_600_000,
            Interval::TwoHours => 7_200_000,
            Interval::FourHours => 14_400_000,
            Interval::SixHours => 21_600_000,
            Interval::EightHours => 28_800_000,
            Interval::TwelveHours => 43_200_000,
            Interval::OneDay => 86_400_000,
            Interval::ThreeDays => 259_200_000,
            Interval::OneWeek => 604_800_000,
            Interval::OneMonth => 2_592_000_000, // 30天近似
        }
    }

    /// 获取连续性检测的容忍度 (毫秒)
    /// 根据时间间隔动态调整，考虑网络延迟和服务器时钟偏差
    fn get_tolerance_milliseconds(&self, interval: &Interval) -> i64 {
        match interval {
            // 秒级K线：允许2秒误差
            Interval::OneSecond => 2_000,
            
            // 分钟级K线：允许5-10秒误差
            Interval::OneMinute => 5_000,
            Interval::ThreeMinutes => 8_000,
            Interval::FiveMinutes => 10_000,
            Interval::FifteenMinutes => 15_000,
            Interval::ThirtyMinutes => 20_000,
            
            // 小时级K线：允许30秒-2分钟误差
            Interval::OneHour => 30_000,
            Interval::TwoHours => 60_000,
            Interval::FourHours => 90_000,
            Interval::SixHours => 120_000,
            Interval::EightHours => 120_000,
            Interval::TwelveHours => 180_000,
            
            // 日级K线：允许5-10分钟误差
            Interval::OneDay => 300_000,      // 5分钟
            Interval::ThreeDays => 600_000,   // 10分钟
            Interval::OneWeek => 1_800_000,   // 30分钟
            Interval::OneMonth => 3_600_000,  // 1小时
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ContinuityStats {
        self.stats.read().await.clone()
    }

    /// 获取当前维护的交易对数量
    pub async fn get_tracked_pairs_count(&self) -> usize {
        self.last_open_times.read().await.len()
    }
}

impl Default for KlineContinuityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_first_kline_no_gap() {
        let detector = KlineContinuityDetector::new();
        
        let result = detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000, // 2022-01-01 00:00:00
        ).await;
        
        assert!(!result.has_gap);
        assert!(result.expected_next_time.is_none());
    }

    #[tokio::test]
    async fn test_continuous_klines() {
        let detector = KlineContinuityDetector::new();
        
        // 第一条K线
        detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000, // 2022-01-01 00:00:00
        ).await;
        
        // 连续的第二条K线
        let result = detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995260000, // 2022-01-01 00:01:00
        ).await;
        
        assert!(!result.has_gap);
        assert_eq!(result.expected_next_time, Some(1640995260000));
    }

    #[tokio::test]
    async fn test_gap_detection() {
        let detector = KlineContinuityDetector::new();
        
        // 第一条K线
        detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000, // 2022-01-01 00:00:00
        ).await;
        
        // 有间隙的第二条K线 (跳过了2分钟)
        let result = detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995380000, // 2022-01-01 00:03:00
        ).await;
        
        assert!(result.has_gap);
        assert_eq!(result.expected_next_time, Some(1640995260000));
        assert_eq!(result.gap_duration_ms, Some(120_000)); // 2分钟间隙
        assert_eq!(result.data_quality, DataQuality::Suspect);
    }

    #[tokio::test]
    async fn test_tolerance_boundary() {
        let detector = KlineContinuityDetector::new();
        
        // 第一条K线
        detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000, // 2022-01-01 00:00:00
        ).await;
        
        // 在容忍范围内的延迟 (3秒延迟，容忍度5秒)
        let result = detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995263000, // 2022-01-01 00:01:03
        ).await;
        
        assert!(!result.has_gap); // 应该不认为是间隙
        assert_eq!(result.data_quality, DataQuality::Normal);
        
        // 超出容忍范围的延迟 (10秒延迟，容忍度5秒)
        let result2 = detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995330000, // 2022-01-01 00:02:10
        ).await;
        
        assert!(result2.has_gap); // 应该认为是间隙
        assert_eq!(result2.data_quality, DataQuality::Suspect);
    }

    #[tokio::test]
    async fn test_early_arrival_detection() {
        let detector = KlineContinuityDetector::new();
        
        // 第一条K线
        detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000, // 2022-01-01 00:00:00
        ).await;
        
        // 提前到达的K线 (提前10秒，超出5秒容忍度)
        let result = detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995250000, // 2022-01-01 00:00:50 (提前10秒)
        ).await;
        
        assert!(result.has_gap); // 应该检测到异常
        assert_eq!(result.data_quality, DataQuality::Suspect);
        assert!(result.gap_duration_ms.unwrap() < 0); // 负值表示提前
    }

    #[tokio::test]
    async fn test_different_symbols_independent() {
        let detector = KlineContinuityDetector::new();
        
        // BTCUSDT
        detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000,
        ).await;
        
        // ETHUSDT (独立跟踪)
        let result = detector.check_continuity(
            Exchange::Binance,
            "ETHUSDT",
            Interval::OneMinute,
            1640995200000,
        ).await;
        
        assert!(!result.has_gap); // 第一条K线不应该有间隙
    }

    #[tokio::test]
    async fn test_stats_update() {
        let detector = KlineContinuityDetector::new();
        
        let initial_stats = detector.get_stats().await;
        assert_eq!(initial_stats.total_checks, 0);
        assert_eq!(initial_stats.gaps_detected, 0);
        
        // 执行一次检查
        detector.check_continuity(
            Exchange::Binance,
            "BTCUSDT",
            Interval::OneMinute,
            1640995200000,
        ).await;
        
        let stats = detector.get_stats().await;
        assert_eq!(stats.total_checks, 1);
        assert_eq!(stats.gaps_detected, 0);
        assert!(stats.last_check_time.is_some());
    }
}