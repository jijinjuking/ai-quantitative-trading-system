use anyhow::Result;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, IntCounter,
    IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 指标收集器
pub struct MetricsCollector {
    registry: Registry,
    counters: HashMap<String, Counter>,
    counter_vecs: HashMap<String, CounterVec>,
    int_counters: HashMap<String, IntCounter>,
    int_counter_vecs: HashMap<String, IntCounterVec>,
    gauges: HashMap<String, Gauge>,
    gauge_vecs: HashMap<String, GaugeVec>,
    int_gauges: HashMap<String, IntGauge>,
    int_gauge_vecs: HashMap<String, IntGaugeVec>,
    histograms: HashMap<String, Histogram>,
    histogram_vecs: HashMap<String, HistogramVec>,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            counters: HashMap::new(),
            counter_vecs: HashMap::new(),
            int_counters: HashMap::new(),
            int_counter_vecs: HashMap::new(),
            gauges: HashMap::new(),
            gauge_vecs: HashMap::new(),
            int_gauges: HashMap::new(),
            int_gauge_vecs: HashMap::new(),
            histograms: HashMap::new(),
            histogram_vecs: HashMap::new(),
        }
    }

    /// 注册计数器
    pub fn register_counter(&mut self, name: &str, help: &str) -> Result<()> {
        let opts = Opts::new(name, help);
        let counter = Counter::with_opts(opts)?;
        self.registry.register(Box::new(counter.clone()))?;
        self.counters.insert(name.to_string(), counter);
        Ok(())
    }

    /// 注册计数器向量
    pub fn register_counter_vec(&mut self, name: &str, help: &str, labels: &[&str]) -> Result<()> {
        let opts = Opts::new(name, help);
        let counter_vec = CounterVec::new(opts, labels)?;
        self.registry.register(Box::new(counter_vec.clone()))?;
        self.counter_vecs.insert(name.to_string(), counter_vec);
        Ok(())
    }

    /// 注册整数计数器
    pub fn register_int_counter(&mut self, name: &str, help: &str) -> Result<()> {
        let opts = Opts::new(name, help);
        let int_counter = IntCounter::with_opts(opts)?;
        self.registry.register(Box::new(int_counter.clone()))?;
        self.int_counters.insert(name.to_string(), int_counter);
        Ok(())
    }

    /// 注册整数计数器向量
    pub fn register_int_counter_vec(&mut self, name: &str, help: &str, labels: &[&str]) -> Result<()> {
        let opts = Opts::new(name, help);
        let int_counter_vec = IntCounterVec::new(opts, labels)?;
        self.registry.register(Box::new(int_counter_vec.clone()))?;
        self.int_counter_vecs.insert(name.to_string(), int_counter_vec);
        Ok(())
    }

    /// 注册仪表
    pub fn register_gauge(&mut self, name: &str, help: &str) -> Result<()> {
        let opts = Opts::new(name, help);
        let gauge = Gauge::with_opts(opts)?;
        self.registry.register(Box::new(gauge.clone()))?;
        self.gauges.insert(name.to_string(), gauge);
        Ok(())
    }

    /// 注册仪表向量
    pub fn register_gauge_vec(&mut self, name: &str, help: &str, labels: &[&str]) -> Result<()> {
        let opts = Opts::new(name, help);
        let gauge_vec = GaugeVec::new(opts, labels)?;
        self.registry.register(Box::new(gauge_vec.clone()))?;
        self.gauge_vecs.insert(name.to_string(), gauge_vec);
        Ok(())
    }

    /// 注册整数仪表
    pub fn register_int_gauge(&mut self, name: &str, help: &str) -> Result<()> {
        let opts = Opts::new(name, help);
        let int_gauge = IntGauge::with_opts(opts)?;
        self.registry.register(Box::new(int_gauge.clone()))?;
        self.int_gauges.insert(name.to_string(), int_gauge);
        Ok(())
    }

    /// 注册整数仪表向量
    pub fn register_int_gauge_vec(&mut self, name: &str, help: &str, labels: &[&str]) -> Result<()> {
        let opts = Opts::new(name, help);
        let int_gauge_vec = IntGaugeVec::new(opts, labels)?;
        self.registry.register(Box::new(int_gauge_vec.clone()))?;
        self.int_gauge_vecs.insert(name.to_string(), int_gauge_vec);
        Ok(())
    }

    /// 注册直方图
    pub fn register_histogram(&mut self, name: &str, help: &str, buckets: Vec<f64>) -> Result<()> {
        let opts = HistogramOpts::new(name, help).buckets(buckets);
        let histogram = Histogram::with_opts(opts)?;
        self.registry.register(Box::new(histogram.clone()))?;
        self.histograms.insert(name.to_string(), histogram);
        Ok(())
    }

    /// 注册直方图向量
    pub fn register_histogram_vec(
        &mut self,
        name: &str,
        help: &str,
        labels: &[&str],
        buckets: Vec<f64>,
    ) -> Result<()> {
        let opts = HistogramOpts::new(name, help).buckets(buckets);
        let histogram_vec = HistogramVec::new(opts, labels)?;
        self.registry.register(Box::new(histogram_vec.clone()))?;
        self.histogram_vecs.insert(name.to_string(), histogram_vec);
        Ok(())
    }

    /// 增加计数器
    pub fn inc_counter(&self, name: &str) -> Result<()> {
        if let Some(counter) = self.counters.get(name) {
            counter.inc();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Counter '{}' not found", name))
        }
    }

    /// 增加计数器指定值
    pub fn inc_counter_by(&self, name: &str, value: f64) -> Result<()> {
        if let Some(counter) = self.counters.get(name) {
            counter.inc_by(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Counter '{}' not found", name))
        }
    }

    /// 增加计数器向量
    pub fn inc_counter_vec(&self, name: &str, labels: &[&str]) -> Result<()> {
        if let Some(counter_vec) = self.counter_vecs.get(name) {
            counter_vec.with_label_values(labels).inc();
            Ok(())
        } else {
            Err(anyhow::anyhow!("CounterVec '{}' not found", name))
        }
    }

    /// 设置仪表值
    pub fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.set(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Gauge '{}' not found", name))
        }
    }

    /// 设置仪表向量值
    pub fn set_gauge_vec(&self, name: &str, labels: &[&str], value: f64) -> Result<()> {
        if let Some(gauge_vec) = self.gauge_vecs.get(name) {
            gauge_vec.with_label_values(labels).set(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("GaugeVec '{}' not found", name))
        }
    }

    /// 观察直方图值
    pub fn observe_histogram(&self, name: &str, value: f64) -> Result<()> {
        if let Some(histogram) = self.histograms.get(name) {
            histogram.observe(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Histogram '{}' not found", name))
        }
    }

    /// 观察直方图向量值
    pub fn observe_histogram_vec(&self, name: &str, labels: &[&str], value: f64) -> Result<()> {
        if let Some(histogram_vec) = self.histogram_vecs.get(name) {
            histogram_vec.with_label_values(labels).observe(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("HistogramVec '{}' not found", name))
        }
    }

    /// 获取指标文本格式
    pub fn gather(&self) -> Result<String> {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// 获取注册表
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 应用指标
pub struct AppMetrics {
    collector: Arc<MetricsCollector>,
}

impl AppMetrics {
    /// 创建应用指标
    pub fn new() -> Result<Self> {
        let mut collector = MetricsCollector::new();

        // 注册基础指标
        collector.register_int_counter_vec("http_requests_total", "Total HTTP requests", &["method", "path", "status"])?;
        collector.register_histogram_vec("http_request_duration_seconds", "HTTP request duration", &["method", "path"], prometheus::DEFAULT_BUCKETS.to_vec())?;
        collector.register_int_gauge("active_connections", "Active connections")?;
        collector.register_int_counter("database_queries_total", "Total database queries")?;
        collector.register_histogram("database_query_duration_seconds", "Database query duration", prometheus::DEFAULT_BUCKETS.to_vec())?;

        // 交易相关指标
        collector.register_int_counter_vec("orders_total", "Total orders", &["symbol", "side", "status"])?;
        collector.register_counter_vec("trading_volume", "Trading volume", &["symbol", "exchange"])?;
        collector.register_gauge_vec("account_balance", "Account balance", &["user_id", "asset"])?;
        collector.register_int_gauge_vec("active_positions", "Active positions", &["user_id", "symbol"])?;

        // 系统指标
        collector.register_gauge("memory_usage_bytes", "Memory usage in bytes")?;
        collector.register_gauge("cpu_usage_percent", "CPU usage percentage")?;
        collector.register_int_gauge("goroutines_count", "Number of goroutines")?;

        Ok(Self {
            collector: Arc::new(collector),
        })
    }

    /// 记录HTTP请求
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) -> Result<()> {
        self.collector.inc_counter_vec("http_requests_total", &[method, path, &status.to_string()])?;
        self.collector.observe_histogram_vec("http_request_duration_seconds", &[method, path], duration.as_secs_f64())?;
        Ok(())
    }

    /// 设置活跃连接数
    pub fn set_active_connections(&self, count: i64) -> Result<()> {
        self.collector.int_gauges.get("active_connections").unwrap().set(count);
        Ok(())
    }

    /// 记录数据库查询
    pub fn record_database_query(&self, duration: Duration) -> Result<()> {
        self.collector.inc_counter("database_queries_total")?;
        self.collector.observe_histogram("database_query_duration_seconds", duration.as_secs_f64())?;
        Ok(())
    }

    /// 记录订单
    pub fn record_order(&self, symbol: &str, side: &str, status: &str) -> Result<()> {
        self.collector.inc_counter_vec("orders_total", &[symbol, side, status])?;
        Ok(())
    }

    /// 记录交易量
    pub fn record_trading_volume(&self, symbol: &str, exchange: &str, volume: f64) -> Result<()> {
        self.collector.counter_vecs.get("trading_volume").unwrap().with_label_values(&[symbol, exchange]).inc_by(volume);
        Ok(())
    }

    /// 设置账户余额
    pub fn set_account_balance(&self, user_id: &str, asset: &str, balance: f64) -> Result<()> {
        self.collector.set_gauge_vec("account_balance", &[user_id, asset], balance)?;
        Ok(())
    }

    /// 设置活跃仓位
    pub fn set_active_positions(&self, user_id: &str, symbol: &str, count: i64) -> Result<()> {
        self.collector.int_gauge_vecs.get("active_positions").unwrap().with_label_values(&[user_id, symbol]).set(count);
        Ok(())
    }

    /// 设置内存使用量
    pub fn set_memory_usage(&self, bytes: f64) -> Result<()> {
        self.collector.set_gauge("memory_usage_bytes", bytes)?;
        Ok(())
    }

    /// 设置CPU使用率
    pub fn set_cpu_usage(&self, percent: f64) -> Result<()> {
        self.collector.set_gauge("cpu_usage_percent", percent)?;
        Ok(())
    }

    /// 获取指标文本
    pub fn gather(&self) -> Result<String> {
        self.collector.gather()
    }

    /// 获取收集器
    pub fn collector(&self) -> Arc<MetricsCollector> {
        self.collector.clone()
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// 性能计时器
pub struct Timer {
    start: Instant,
    name: String,
    labels: Vec<String>,
}

impl Timer {
    /// 创建新的计时器
    pub fn new(name: String, labels: Vec<String>) -> Self {
        Self {
            start: Instant::now(),
            name,
            labels,
        }
    }

    /// 开始计时
    pub fn start(name: &str) -> Self {
        Self::new(name.to_string(), Vec::new())
    }

    /// 带标签开始计时
    pub fn start_with_labels(name: &str, labels: Vec<&str>) -> Self {
        Self::new(name.to_string(), labels.iter().map(|s| s.to_string()).collect())
    }

    /// 停止计时并记录
    pub fn stop(self, metrics: &AppMetrics) -> Duration {
        let duration = self.start.elapsed();
        
        if self.labels.is_empty() {
            let _ = metrics.collector.observe_histogram(&self.name, duration.as_secs_f64());
        } else {
            let label_refs: Vec<&str> = self.labels.iter().map(|s| s.as_str()).collect();
            let _ = metrics.collector.observe_histogram_vec(&self.name, &label_refs, duration.as_secs_f64());
        }

        duration
    }
}

/// 指标中间件
pub struct MetricsMiddleware {
    metrics: Arc<AppMetrics>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<AppMetrics>) -> Self {
        Self { metrics }
    }

    /// 记录HTTP请求指标
    pub async fn record_request<F, Fut>(&self, method: &str, path: &str, handler: F) -> Result<axum::response::Response>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<axum::response::Response>>,
    {
        let timer = Timer::start_with_labels("http_request_duration_seconds", vec![method, path]);
        
        let result = handler().await;
        
        let duration = timer.stop(&self.metrics);
        
        match &result {
            Ok(response) => {
                let status = response.status().as_u16();
                let _ = self.metrics.record_http_request(method, path, status, duration);
            }
            Err(_) => {
                let _ = self.metrics.record_http_request(method, path, 500, duration);
            }
        }

        result
    }
}

/// 系统指标收集器
pub struct SystemMetricsCollector {
    metrics: Arc<AppMetrics>,
}

impl SystemMetricsCollector {
    pub fn new(metrics: Arc<AppMetrics>) -> Self {
        Self { metrics }
    }

    /// 收集系统指标
    pub async fn collect_system_metrics(&self) -> Result<()> {
        // 收集内存使用量
        if let Ok(memory) = self.get_memory_usage().await {
            let _ = self.metrics.set_memory_usage(memory);
        }

        // 收集CPU使用率
        if let Ok(cpu) = self.get_cpu_usage().await {
            let _ = self.metrics.set_cpu_usage(cpu);
        }

        Ok(())
    }

    /// 获取内存使用量
    async fn get_memory_usage(&self) -> Result<f64> {
        // 这里应该实现实际的内存使用量获取逻辑
        // 为了简化，返回模拟值
        Ok(1024.0 * 1024.0 * 100.0) // 100MB
    }

    /// 获取CPU使用率
    async fn get_cpu_usage(&self) -> Result<f64> {
        // 这里应该实现实际的CPU使用率获取逻辑
        // 为了简化，返回模拟值
        Ok(25.5) // 25.5%
    }

    /// 启动定期收集
    pub async fn start_periodic_collection(&self, interval: Duration) {
        let mut interval_timer = tokio::time::interval(interval);
        
        loop {
            interval_timer.tick().await;
            if let Err(e) = self.collect_system_metrics().await {
                tracing::error!("Failed to collect system metrics: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        
        // 注册指标
        collector.register_counter("test_counter", "Test counter").unwrap();
        collector.register_gauge("test_gauge", "Test gauge").unwrap();
        
        // 使用指标
        collector.inc_counter("test_counter").unwrap();
        collector.set_gauge("test_gauge", 42.0).unwrap();
        
        // 获取指标
        let metrics_text = collector.gather().unwrap();
        assert!(metrics_text.contains("test_counter"));
        assert!(metrics_text.contains("test_gauge"));
    }

    #[test]
    fn test_app_metrics() {
        let metrics = AppMetrics::new().unwrap();
        
        // 记录HTTP请求
        metrics.record_http_request("GET", "/api/test", 200, Duration::from_millis(100)).unwrap();
        
        // 记录订单
        metrics.record_order("BTCUSDT", "BUY", "FILLED").unwrap();
        
        // 获取指标
        let metrics_text = metrics.gather().unwrap();
        assert!(metrics_text.contains("http_requests_total"));
        assert!(metrics_text.contains("orders_total"));
    }

    #[test]
    fn test_timer() {
        let metrics = AppMetrics::new().unwrap();
        let timer = Timer::start("test_operation");
        
        // 模拟一些工作
        std::thread::sleep(Duration::from_millis(10));
        
        let duration = timer.stop(&metrics);
        assert!(duration.as_millis() >= 10);
    }
}