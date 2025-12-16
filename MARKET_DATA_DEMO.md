# 市场数据服务演示

## 快速启动

### 1. 启动Docker环境

```powershell
# 构建并启动环境
.\scripts\docker-dev.ps1 build
.\scripts\docker-dev.ps1 up
```

### 2. 编译并运行服务

```powershell
# 进入开发容器
.\scripts\docker-dev.ps1 shell

# 在容器内编译
cargo build --release

# 运行市场数据服务
cargo run --bin market-data
```

### 3. 验证服务

```bash
# 健康检查
curl http://localhost:8000/health

# 查看指标
curl http://localhost:8000/metrics
```

## 服务功能

- ✅ 数据验证和标准化
- ✅ 多存储后端支持
- ✅ 数据连续性管理
- ✅ 健康检查和监控
- 🚧 WebSocket实时推送
- 🚧 币安数据连接

## 监控界面

- Kafka UI: http://localhost:8080
- Redis Commander: http://localhost:8081
- ClickHouse: http://localhost:8123