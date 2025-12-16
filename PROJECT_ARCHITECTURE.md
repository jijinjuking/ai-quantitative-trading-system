# 🏗️ 企业级量化交易平台 - 详细架构设计

## 📋 目录
1. [系统架构概览](#系统架构概览)
2. [微服务设计](#微服务设计)
3. [数据架构](#数据架构)
4. [技术选型详解](#技术选型详解)
5. [部署架构](#部署架构)
6. [安全架构](#安全架构)

## 🎯 系统架构概览

### 整体架构图
```
┌─────────────────────────────────────────────────────────────────┐
│                        前端层 (Frontend Layer)                    │
├─────────────────────────────────────────────────────────────────┤
│  Web应用    │  移动端    │  管理后台   │  API文档   │  监控面板    │
└─────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────┐
│                      API网关层 (Gateway Layer)                   │
├─────────────────────────────────────────────────────────────────┤
│  负载均衡   │  路由转发   │  认证授权   │  限流熔断   │  日志监控    │
└─────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────┐
│                      业务服务层 (Service Layer)                   │
├─────────────────────────────────────────────────────────────────┤
│ 市场数据服务 │ 交易引擎服务 │ 策略引擎服务 │ 风险管理服务 │ 用户管理服务 │
│ 通知服务    │ 分析服务    │ 配置服务    │ 文件服务    │ 审计服务    │
└─────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────┐
│                      数据层 (Data Layer)                         │
├─────────────────────────────────────────────────────────────────┤
│ PostgreSQL  │  Redis     │ ClickHouse  │  Kafka     │  MinIO     │
│ (业务数据)   │ (缓存)      │ (时序数据)   │ (消息队列)  │ (文件存储)  │
└─────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────┐
│                    外部接口层 (External Layer)                    │
├─────────────────────────────────────────────────────────────────┤
│  币安API    │  OKX API   │  火币API    │  邮件服务   │  短信服务    │
└─────────────────────────────────────────────────────────────────┘
```

## 🔧 微服务设计

### 1. API网关服务 (Gateway Service)
**职责**: 统一入口、路由转发、认证授权
**技术栈**: Rust + Axum + JWT
**核心功能**:
- 请求路由和负载均衡
- 身份认证和权限控制
- 限流和熔断保护
- 请求/响应日志记录
- API版本管理

**接口设计**:
```rust
// 路由配置
pub struct RouteConfig {
    pub path: String,
    pub method: HttpMethod,
    pub service: String,
    pub auth_required: bool,
    pub rate_limit: Option<RateLimit>,
}

// 认证中间件
pub struct AuthMiddleware {
    pub jwt_secret: String,
    pub token_expiry: Duration,
}
```

### 2. 市场数据服务 (Market Data Service)
**职责**: 实时数据采集、处理、存储、分发
**技术栈**: Rust + Tokio + WebSocket + ClickHouse
**核心功能**:
- 多交易所WebSocket连接管理
- 数据标准化和清洗
- 实时数据流处理
- 历史数据存储和查询
- 数据订阅和推送

**数据模型**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketTick {
    pub exchange: String,
    pub symbol: String,
    pub timestamp: i64,
    pub price: Decimal,
    pub volume: Decimal,
    pub bid: Decimal,
    pub ask: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Kline {
    pub exchange: String,
    pub symbol: String,
    pub interval: String,
    pub open_time: i64,
    pub close_time: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}
```

### 3. 交易引擎服务 (Trading Engine Service)
**职责**: 订单管理、交易执行、仓位管理
**技术栈**: Rust + 状态机 + 原子操作
**核心功能**:
- 智能订单路由 (Smart Order Routing)
- 订单生命周期管理
- 实时仓位计算
- 交易执行算法
- 滑点控制

**订单状态机**:
```rust
#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,      // 待提交
    Submitted,    // 已提交
    PartialFilled,// 部分成交
    Filled,       // 完全成交
    Cancelled,    // 已取消
    Rejected,     // 被拒绝
    Failed,       // 执行失败
}

pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 4. 策略引擎服务 (Strategy Engine Service)
**职责**: 策略计算、信号生成、回测分析
**技术栈**: Rust + 插件系统 + 并行计算
**核心功能**:
- 策略插件框架
- 实时信号计算
- 历史回测引擎
- 参数优化算法
- 策略性能分析

**策略框架**:
```rust
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn parameters(&self) -> &StrategyParameters;
    
    async fn initialize(&mut self, context: &StrategyContext) -> Result<()>;
    async fn on_tick(&mut self, tick: &MarketTick) -> Result<Vec<Signal>>;
    async fn on_kline(&mut self, kline: &Kline) -> Result<Vec<Signal>>;
    async fn on_order_update(&mut self, order: &Order) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub strategy_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub signal_type: SignalType,
    pub strength: f64,
    pub price: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub metadata: HashMap<String, Value>,
}
```

### 5. 风险管理服务 (Risk Management Service)
**职责**: 实时风控、限额管理、风险监控
**技术栈**: Rust + 规则引擎 + 实时计算
**核心功能**:
- 实时风险检查
- 多维度限额控制
- 风险指标计算
- 自动止损机制
- 风险报告生成

**风险规则引擎**:
```rust
pub trait RiskRule: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> u8;
    
    async fn check(&self, context: &RiskContext) -> Result<RiskDecision>;
}

#[derive(Debug, Clone)]
pub enum RiskDecision {
    Allow,
    Reject { reason: String },
    Modify { new_quantity: Decimal, reason: String },
    Warning { message: String },
}

pub struct RiskLimits {
    pub max_position_size: Decimal,
    pub max_daily_loss: Decimal,
    pub max_drawdown: Decimal,
    pub max_leverage: Decimal,
    pub max_order_value: Decimal,
}
```

### 6. 用户管理服务 (User Management Service)
**职责**: 用户认证、权限管理、账户管理
**技术栈**: Rust + JWT + RBAC + 加密
**核心功能**:
- 用户注册和认证
- 角色权限管理
- API密钥管理
- 多账户支持
- 安全审计

**用户模型**:
```rust
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: UserStatus,
    pub roles: Vec<Role>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange: String,
    pub key: String,
    pub secret: String,
    pub passphrase: Option<String>,
    pub permissions: Vec<Permission>,
    pub is_active: bool,
}
```

## 💾 数据架构

### 数据库选型和用途

#### 1. PostgreSQL (主数据库)
**用途**: 业务数据存储
**数据类型**:
- 用户信息和权限
- 订单和交易记录
- 策略配置和参数
- 风险规则和限额
- 系统配置

**表设计示例**:
```sql
-- 用户表
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 订单表
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL,
    order_type VARCHAR(20) NOT NULL,
    quantity DECIMAL(20,8) NOT NULL,
    price DECIMAL(20,8),
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### 2. Redis (缓存和会话)
**用途**: 高速缓存和实时数据
**数据类型**:
- 用户会话和JWT令牌
- 实时价格缓存
- 限流计数器
- 配置缓存
- 临时数据

**使用模式**:
```rust
// 价格缓存
pub async fn cache_price(redis: &Redis, symbol: &str, price: Decimal) -> Result<()> {
    let key = format!("price:{}", symbol);
    redis.setex(&key, 60, price.to_string()).await
}

// 限流控制
pub async fn check_rate_limit(redis: &Redis, user_id: &str) -> Result<bool> {
    let key = format!("rate_limit:{}", user_id);
    let count: i32 = redis.incr(&key, 1).await?;
    if count == 1 {
        redis.expire(&key, 60).await?;
    }
    Ok(count <= 100) // 每分钟100次
}
```

#### 3. ClickHouse (时序数据)
**用途**: 高频时序数据存储
**数据类型**:
- K线数据
- Tick数据
- 交易执行记录
- 性能指标
- 日志数据

**表结构**:
```sql
-- K线数据表
CREATE TABLE klines (
    exchange String,
    symbol String,
    interval String,
    open_time DateTime64(3),
    close_time DateTime64(3),
    open Decimal64(8),
    high Decimal64(8),
    low Decimal64(8),
    close Decimal64(8),
    volume Decimal64(8),
    quote_volume Decimal64(8),
    trades_count UInt32
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(open_time)
ORDER BY (exchange, symbol, interval, open_time);

-- Tick数据表
CREATE TABLE ticks (
    exchange String,
    symbol String,
    timestamp DateTime64(3),
    price Decimal64(8),
    volume Decimal64(8),
    side String
) ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (exchange, symbol, timestamp);
```

#### 4. Apache Kafka (消息队列)
**用途**: 异步消息传递和事件流
**Topic设计**:
- `market-data`: 市场数据流
- `trading-signals`: 交易信号
- `order-events`: 订单事件
- `risk-alerts`: 风险告警
- `user-events`: 用户事件

**消息格式**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataEvent {
    pub event_type: String,
    pub exchange: String,
    pub symbol: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingSignalEvent {
    pub signal_id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub side: String,
    pub strength: f64,
    pub timestamp: i64,
}
```

## 🔧 技术选型详解

### 后端技术栈

#### 1. Rust语言优势
- **性能**: 零成本抽象，接近C++性能
- **安全**: 内存安全，避免空指针和缓冲区溢出
- **并发**: 优秀的异步编程支持
- **生态**: 丰富的金融和网络库

#### 2. Axum Web框架
- **异步**: 基于Tokio的高性能异步框架
- **类型安全**: 编译时类型检查
- **中间件**: 丰富的中间件生态
- **WebSocket**: 原生WebSocket支持

#### 3. 数据库驱动
```toml
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "decimal"] }
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
clickhouse = { version = "0.11", features = ["time", "uuid"] }
rdkafka = { version = "0.36", features = ["cmake-build", "ssl-vendored"] }
```

### 前端技术栈

#### 1. Vue 3 + TypeScript
```json
{
  "dependencies": {
    "vue": "^3.3.0",
    "vue-router": "^4.2.0",
    "@vueuse/core": "^10.0.0",
    "pinia": "^2.1.0",
    "element-plus": "^2.3.0",
    "axios": "^1.4.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.2.0",
    "typescript": "^5.0.0",
    "vite": "^4.3.0"
  }
}
```

#### 2. TradingView图表集成
```typescript
// TradingView配置
export interface ChartConfig {
  symbol: string;
  interval: string;
  container: string;
  library_path: string;
  datafeed: IBasicDataFeed;
  theme: 'light' | 'dark';
  timezone: string;
}
```

## 🚀 部署架构

### Docker容器化
```yaml
# docker-compose.yml
version: '3.8'
services:
  gateway:
    build: ./services/gateway
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
    depends_on:
      - redis
      - postgres

  market-data:
    build: ./services/market-data
    environment:
      - CLICKHOUSE_URL=clickhouse://clickhouse:9000
      - KAFKA_BROKERS=kafka:9092
    depends_on:
      - clickhouse
      - kafka

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: trading_platform
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  clickhouse:
    image: clickhouse/clickhouse-server:latest
    environment:
      CLICKHOUSE_DB: market_data
    volumes:
      - clickhouse_data:/var/lib/clickhouse

volumes:
  postgres_data:
  redis_data:
  clickhouse_data:
```

### Kubernetes部署
```yaml
# k8s/gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: gateway
  template:
    metadata:
      labels:
        app: gateway
    spec:
      containers:
      - name: gateway
        image: trading-platform/gateway:latest
        ports:
        - containerPort: 8080
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## 🔒 安全架构

### 1. 认证和授权
```rust
// JWT认证
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // 用户ID
    pub exp: usize,     // 过期时间
    pub iat: usize,     // 签发时间
    pub roles: Vec<String>, // 用户角色
}

// RBAC权限控制
pub struct Permission {
    pub resource: String,
    pub action: String,
}

pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
}
```

### 2. 数据加密
```rust
// API密钥加密存储
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        // AES-256-GCM加密
    }
    
    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        // AES-256-GCM解密
    }
}
```

### 3. 网络安全
- TLS 1.3加密传输
- API限流和DDoS防护
- IP白名单控制
- 请求签名验证

---

这个架构设计确保了系统的高性能、高可用性、可扩展性和安全性，为企业级量化交易提供了坚实的技术基础。