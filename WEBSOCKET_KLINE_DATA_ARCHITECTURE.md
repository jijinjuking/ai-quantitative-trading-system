# 专业量化交易系统 - WebSocket K线数据架构开发大纲

## 🚀 核心原则
- **禁止使用HTTP API获取K线数据** - 有频率限制，延迟高
- **强制使用WebSocket实时数据流** - 无限制，毫秒级延迟
- **多交易所并发连接** - 币安、OKX、火币等
- **高可用性设计** - 断线重连、数据完整性保证

## 📊 WebSocket数据流架构

### 1. 数据源连接层 (WebSocket Connectors)
```
币安WebSocket: wss://stream.binance.com:9443/ws
OKX WebSocket: wss://ws.okx.com:8443/ws/v5/public
火币WebSocket: wss://api.huobi.pro/ws
```

### 2. 实时数据类型
- **K线数据流**: `symbol@kline_1m`, `symbol@kline_5m`, `symbol@kline_15m`, `symbol@kline_1h`, `symbol@kline_4h`, `symbol@kline_1d`
- **Ticker数据流**: `symbol@ticker` (24小时统计)
- **深度数据流**: `symbol@depth20@100ms` (订单簿)
- **成交数据流**: `symbol@trade` (实时成交)

### 3. 数据处理管道
```
WebSocket原始数据 → 数据标准化 → 数据验证 → 存储分发 → 策略引擎
```

## 🏗️ 技术实现架构

### 1. WebSocket连接管理器
```rust
// services/market-data/src/websocket/connection_manager.rs
pub struct WebSocketConnectionManager {
    connections: HashMap<String, WebSocketConnection>,
    reconnect_strategy: ReconnectStrategy,
    health_monitor: HealthMonitor,
}
```

### 2. 数据流订阅管理
```rust
// 多交易对批量订阅
let streams = vec![
    "btcusdt@kline_1m",
    "ethusdt@kline_1m", 
    "adausdt@kline_1m",
    // ... 支持1000+交易对
];
```

### 3. 实时数据存储
- **Redis**: 最新K线数据缓存 (毫秒级读取)
- **ClickHouse**: 历史K线数据存储 (时序数据库)
- **Kafka**: 数据流分发 (解耦生产消费)

## 🔄 数据流处理流程

### 1. WebSocket数据接收
```rust
async fn handle_websocket_message(&self, message: Message) -> Result<()> {
    match message {
        Message::Text(data) => {
            let kline_data: BinanceKlineData = serde_json::from_str(&data)?;
            self.process_kline_data(kline_data).await?;
        }
        Message::Ping(ping) => {
            self.send_pong(ping).await?;
        }
        _ => {}
    }
    Ok(())
}
```

### 2. 数据标准化处理
```rust
pub struct StandardKline {
    pub exchange: String,      // "binance"
    pub symbol: String,        // "BTCUSDT"
    pub interval: String,      // "1m"
    pub open_time: i64,        // 开盘时间戳
    pub close_time: i64,       // 收盘时间戳
    pub open: Decimal,         // 开盘价
    pub high: Decimal,         // 最高价
    pub low: Decimal,          // 最低价
    pub close: Decimal,        // 收盘价
    pub volume: Decimal,       // 成交量
    pub quote_volume: Decimal, // 成交额
    pub trade_count: u64,      // 成交笔数
    pub is_closed: bool,       // K线是否完结
}
```

### 3. 实时数据分发
```rust
// 发送到Kafka主题
producer.send_record("kline-data", &standard_kline).await?;

// 更新Redis缓存
redis.hset(
    format!("kline:{}:{}", symbol, interval),
    "latest",
    serde_json::to_string(&standard_kline)?
).await?;

// 存储到ClickHouse
clickhouse.insert_kline(&standard_kline).await?;
```

## 🛡️ 高可用性保证

### 1. 断线重连机制
```rust
pub struct ReconnectStrategy {
    max_retries: u32,           // 最大重试次数
    initial_delay: Duration,    // 初始延迟
    max_delay: Duration,        // 最大延迟
    backoff_multiplier: f64,    // 退避倍数
}
```

### 2. 数据完整性检查
- **时间戳连续性检查**: 检测数据缺失
- **价格合理性检查**: 检测异常价格
- **成交量合理性检查**: 检测异常成交量

### 3. 健康监控
```rust
pub struct HealthMetrics {
    pub connection_status: ConnectionStatus,
    pub last_message_time: i64,
    pub messages_per_second: f64,
    pub error_rate: f64,
    pub latency_ms: u64,
}
```

## 📈 性能优化策略

### 1. 连接池管理
- **单连接多流**: 一个WebSocket连接订阅多个数据流
- **负载均衡**: 多个连接分担数据流负载
- **智能路由**: 根据延迟选择最优连接

### 2. 数据压缩
- **启用gzip压缩**: 减少网络传输量
- **二进制协议**: 使用MessagePack等高效序列化

### 3. 内存优化
- **对象池**: 复用数据结构对象
- **批量处理**: 批量写入数据库
- **异步处理**: 非阻塞数据处理

## 🔧 配置管理

### 1. WebSocket配置
```toml
[websocket]
# 币安配置
[websocket.binance]
url = "wss://stream.binance.com:9443/ws"
max_connections = 10
reconnect_delay = 5000
ping_interval = 30000
symbols = ["BTCUSDT", "ETHUSDT", "ADAUSDT"]
intervals = ["1m", "5m", "15m", "1h", "4h", "1d"]

# OKX配置  
[websocket.okx]
url = "wss://ws.okx.com:8443/ws/v5/public"
max_connections = 5
reconnect_delay = 3000
ping_interval = 25000
```

### 2. 存储配置
```toml
[storage]
# Redis配置
[storage.redis]
url = "redis://localhost:6379"
db = 0
max_connections = 20

# ClickHouse配置
[storage.clickhouse]
url = "http://localhost:8123"
database = "trading_data"
table = "klines"
batch_size = 1000
```

## 🚦 监控告警

### 1. 关键指标监控
- **连接状态**: WebSocket连接健康度
- **数据延迟**: 数据接收延迟
- **数据完整性**: 缺失数据比例
- **处理性能**: 每秒处理消息数

### 2. 告警规则
- **连接断开**: 立即告警
- **数据延迟 > 1秒**: 警告告警
- **数据缺失 > 1%**: 严重告警
- **处理延迟 > 100ms**: 性能告警

## 🎯 开发优先级

### Phase 1: 核心WebSocket连接 (1周)
- [ ] WebSocket连接管理器
- [ ] 币安K线数据流接收
- [ ] 基础数据标准化
- [ ] Redis缓存存储

### Phase 2: 多交易所支持 (1周)
- [ ] OKX WebSocket连接
- [ ] 火币WebSocket连接
- [ ] 统一数据格式处理
- [ ] ClickHouse历史存储

### Phase 3: 高可用性 (1周)
- [ ] 断线重连机制
- [ ] 数据完整性检查
- [ ] 健康监控系统
- [ ] 性能优化

### Phase 4: 监控告警 (3天)
- [ ] Prometheus指标采集
- [ ] Grafana监控面板
- [ ] 告警规则配置
- [ ] 日志系统完善

## 🔥 关键技术要点

1. **绝对禁止HTTP轮询**: 只能用WebSocket实时流
2. **毫秒级延迟要求**: 数据处理延迟 < 10ms
3. **高并发处理**: 支持1000+交易对同时订阅
4. **数据一致性**: 确保K线数据完整无缺失
5. **故障恢复**: 3秒内自动重连恢复

这就是专业量化交易系统的WebSocket数据架构！🚀