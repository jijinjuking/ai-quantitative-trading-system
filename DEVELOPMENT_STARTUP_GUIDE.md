# 🚀 开发阶段启动指南 - 数据连续性保障

## 🎯 开发阶段常见场景处理

### 场景1: 每日开发启动
```bash
# 1. 启动基础设施
docker-compose up -d redis clickhouse kafka

# 2. 检查服务状态
./scripts/check-infrastructure.sh

# 3. 启动市场数据服务（自动恢复状态）
cargo run --bin market-data-service

# 输出示例：
# 2024-01-01 09:00:00 INFO  Loading previous service state...
# 2024-01-01 09:00:01 INFO  Service state loaded from file: ./data/service_state.json
# 2024-01-01 09:00:02 INFO  Checking data consistency for 50 symbols...
# 2024-01-01 09:00:03 WARN  Data gap detected for BTCUSDT: 8 hours
# 2024-01-01 09:00:04 INFO  Filling data gap from REST API...
# 2024-01-01 09:00:10 INFO  Successfully filled 480 records for BTCUSDT
# 2024-01-01 09:00:11 INFO  Cache preload completed: 50 symbols
# 2024-01-01 09:00:12 INFO  🚀 Market Data Service ready!
```

### 场景2: 代码修改重启
```bash
# 1. 优雅关机（Ctrl+C）
^C
# 输出：
# 2024-01-01 12:30:00 INFO  Received Ctrl+C, initiating graceful shutdown
# 2024-01-01 12:30:01 INFO  Stopping data ingestion...
# 2024-01-01 12:30:02 INFO  Flushing all buffers...
# 2024-01-01 12:30:03 INFO  Saving service state...
# 2024-01-01 12:30:04 INFO  Graceful shutdown completed

# 2. 修改代码...

# 3. 重新启动（从上次状态继续）
cargo run --bin market-data-service
# 输出：
# 2024-01-01 12:35:00 INFO  Resuming from previous state...
# 2024-01-01 12:35:01 INFO  Last processed: BTCUSDT at 12:30:00
# 2024-01-01 12:35:02 INFO  Kafka consumer resuming from offset 123456
# 2024-01-01 12:35:03 INFO  No data gaps detected
# 2024-01-01 12:35:04 INFO  🚀 Service resumed successfully!
```

### 场景3: 网络中断恢复
```bash
# 网络恢复后，服务自动检测并修复
# 输出：
# 2024-01-01 15:45:00 WARN  WebSocket connection lost to binance
# 2024-01-01 15:45:01 INFO  Attempting reconnection...
# 2024-01-01 15:47:00 INFO  WebSocket reconnected to binance
# 2024-01-01 15:47:01 WARN  Data gap detected: 2 minutes
# 2024-01-01 15:47:02 INFO  Filling gap from REST API...
# 2024-01-01 15:47:05 INFO  Gap filled: 120 records
# 2024-01-01 15:47:06 INFO  Data continuity restored
```

## 🛠️ 开发工具和脚本

### 1. 基础设施检查脚本
```bash
# scripts/check-infrastructure.sh
#!/bin/bash

echo "🔍 Checking infrastructure status..."

# 检查Redis
if redis-cli ping > /dev/null 2>&1; then
    echo "✅ Redis: Running"
else
    echo "❌ Redis: Not running"
    exit 1
fi

# 检查ClickHouse
if curl -s http://localhost:8123/ping > /dev/null; then
    echo "✅ ClickHouse: Running"
else
    echo "❌ ClickHouse: Not running"
    exit 1
fi

# 检查Kafka
if docker ps | grep kafka > /dev/null; then
    echo "✅ Kafka: Running"
else
    echo "❌ Kafka: Not running"
    exit 1
fi

echo "🎉 All infrastructure services are running!"
```

### 2. 数据一致性检查脚本
```bash
# scripts/check-data-consistency.sh
#!/bin/bash

echo "🔍 Checking data consistency..."

# 调用服务API检查一致性
response=$(curl -s http://localhost:8081/api/v1/admin/consistency-check)

if echo "$response" | jq -r '.success' | grep -q true; then
    echo "✅ Data consistency check passed"
    echo "$response" | jq -r '.data'
else
    echo "❌ Data consistency issues found"
    echo "$response" | jq -r '.error'
    
    # 自动修复
    echo "🔧 Attempting auto-repair..."
    repair_response=$(curl -s -X POST http://localhost:8081/api/v1/admin/repair-data)
    echo "$repair_response" | jq -r '.data'
fi
```

### 3. 开发环境重置脚本
```bash
# scripts/reset-dev-environment.sh
#!/bin/bash

echo "🔄 Resetting development environment..."

# 1. 停止服务
pkill -f market-data-service

# 2. 清理数据
rm -f ./data/service_state.json
redis-cli FLUSHDB
echo "DROP DATABASE IF EXISTS market_data" | clickhouse-client

# 3. 重新创建数据库
echo "CREATE DATABASE market_data" | clickhouse-client

# 4. 重启基础设施
docker-compose restart

echo "✅ Development environment reset complete"
echo "💡 Run 'cargo run --bin market-data-service' to start fresh"
```

## 📊 监控和调试工具

### 1. 实时状态监控
```bash
# scripts/monitor-service.sh
#!/bin/bash

while true; do
    clear
    echo "📊 Market Data Service Status - $(date)"
    echo "=================================="
    
    # 服务健康状态
    health=$(curl -s http://localhost:8081/health/detailed)
    echo "🏥 Health: $(echo "$health" | jq -r '.data.status')"
    
    # 处理统计
    stats=$(curl -s http://localhost:8081/api/v1/admin/stats)
    echo "📈 Events processed: $(echo "$stats" | jq -r '.data.total_events_processed')"
    echo "⚡ Events/sec: $(echo "$stats" | jq -r '.data.events_per_second')"
    echo "❌ Error rate: $(echo "$stats" | jq -r '.data.error_rate')%"
    
    # 连续性统计
    continuity=$(curl -s http://localhost:8081/api/v1/admin/continuity-stats)
    echo "🔗 Gaps detected: $(echo "$continuity" | jq -r '.data.total_gaps_detected')"
    echo "🔧 Gaps filled: $(echo "$continuity" | jq -r '.data.total_gaps_filled')"
    
    sleep 5
done
```

### 2. 数据间隙检查工具
```bash
# scripts/check-gaps.sh
#!/bin/bash

symbol=${1:-"BTCUSDT"}
hours=${2:-24}

echo "🔍 Checking data gaps for $symbol in last $hours hours..."

# 计算时间范围
end_time=$(date +%s)000
start_time=$((end_time - hours * 3600 * 1000))

# 检查K线数据完整性
for interval in "1m" "5m" "15m" "1h"; do
    echo "Checking $interval klines..."
    
    # 查询ClickHouse
    query="SELECT COUNT(*) as count, MIN(open_time) as min_time, MAX(open_time) as max_time 
           FROM market_klines 
           WHERE symbol='$symbol' AND interval='$interval' 
           AND open_time BETWEEN $start_time AND $end_time"
    
    result=$(echo "$query" | clickhouse-client --format=JSON)
    count=$(echo "$result" | jq -r '.data[0].count')
    
    echo "  📊 $interval: $count records"
done
```

## 🔧 开发配置文件

### 1. 开发环境配置
```toml
# config/development.toml
[server]
host = "0.0.0.0"
port = 8081

[data_processing]
batch_size = 100          # 小批量便于调试
flush_interval = 5        # 5秒刷新
max_gap_seconds = 60      # 1分钟间隙检测

[monitoring]
metrics_enabled = true
health_check_interval = 10  # 10秒健康检查

[continuity]
enable_gap_detection = true
enable_auto_repair = true
preload_cache_on_startup = true
save_state_interval = 30    # 30秒保存状态

[[exchanges.binance]]
enabled = true
symbols = ["BTCUSDT", "ETHUSDT"]  # 开发时只监控少量交易对
```

### 2. Docker Compose开发配置
```yaml
# docker-compose.dev.yml
version: '3.8'
services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - ./data/redis:/data
    command: redis-server --appendonly yes

  clickhouse:
    image: clickhouse/clickhouse-server:latest
    ports:
      - "8123:8123"
      - "9000:9000"
    volumes:
      - ./data/clickhouse:/var/lib/clickhouse
    environment:
      CLICKHOUSE_DB: market_data

  kafka:
    image: confluentinc/cp-kafka:latest
    ports:
      - "9092:9092"
    environment:
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
    depends_on:
      - zookeeper

  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
```

## 🎯 开发最佳实践

### 1. 每日开发流程
```bash
# 1. 启动开发环境
./scripts/start-dev-environment.sh

# 2. 检查基础设施
./scripts/check-infrastructure.sh

# 3. 启动服务
cargo run --bin market-data-service

# 4. 开发调试...

# 5. 优雅关机（保存状态）
# Ctrl+C

# 6. 检查数据完整性
./scripts/check-data-consistency.sh
```

### 2. 问题排查流程
```bash
# 1. 检查服务状态
curl http://localhost:8081/health/detailed | jq

# 2. 检查数据连续性
curl http://localhost:8081/api/v1/admin/continuity-stats | jq

# 3. 检查特定交易对
./scripts/check-gaps.sh BTCUSDT 1

# 4. 手动修复数据
curl -X POST http://localhost:8081/api/v1/admin/repair-data \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTCUSDT"}'

# 5. 重置环境（如果需要）
./scripts/reset-dev-environment.sh
```

### 3. 测试数据完整性
```bash
# 模拟网络中断
sudo iptables -A OUTPUT -d api.binance.com -j DROP

# 等待30秒...

# 恢复网络
sudo iptables -D OUTPUT -d api.binance.com -j DROP

# 检查自动修复
tail -f logs/market-data.log | grep -E "(gap|fill|repair)"
```

## 🏆 总结

通过这套完整的数据连续性解决方案，开发阶段可以：

1. **零数据丢失**：Kafka offset管理 + 状态持久化
2. **自动恢复**：间隙检测 + REST API补齐  
3. **快速启动**：状态恢复 + 缓存预热
4. **开发友好**：优雅关机 + 调试工具
5. **问题排查**：完整的监控和修复工具

这样就能确保开发过程中的数据连续性和系统稳定性！