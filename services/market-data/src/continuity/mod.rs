// Phase 1: 只保留必要的kline_detector模块
pub mod kline_detector;

pub use kline_detector::{ContinuityCheckResult, ContinuityStats, KlineContinuityDetector};

// Phase 1: 最小实现，不包含复杂的管理器
