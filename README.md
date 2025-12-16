# AI量化交易系统

## 📋 项目概述

这是一个基于Rust微服务架构的专业量化交易平台，支持实时数据采集、策略执行、风险控制等功能。

## 🏗️ 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Gateway   │    │  Market Data    │
│   (Vue3 + TS)   │◄──►│   (Rust)        │◄──►│   Service       │
└─────────────────┘    └─────────────────┘    │   (Rust)        │
                                              └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │ Trading Engine  │
                       │   (Rust)        │
                       └─────────────────┘
```

## 🚀 核心功能

### ✅ 已完成功能
- **实时数据采集**: 币安WebSocket API集成
- **K线连续性检测**: Phase 1完成，支持间隙检测
- **数据质量标记**: DataQuality系统集成
- **专业交易界面**: 仿币安风格前端
- **微服务架构**: Gateway + Market Data + Trading Engine

### 🔄 开发中功能
- 策略系统完善
- 多交易所支持
- 高级风险控制

## 📁 项目结构

```
23/
├── services/           # 微服务
│   ├── gateway/       # API网关
│   ├── market-data/   # 市场数据服务
│   └── trading-engine/# 交易引擎
├── frontend/          # 前端界面
├── shared/            # 共享库
│   ├── models/        # 数据模型
│   ├── utils/         # 工具库
│   └── protocols/     # 协议定义
└── infrastructure/    # 基础设施配置
```

## 🛠️ 技术栈

- **后端**: Rust + Tokio + Axum
- **前端**: Vue 3 + TypeScript + ECharts
- **数据库**: ClickHouse + Redis
- **消息队列**: Kafka
- **部署**: Docker + Kubernetes

## 📊 数据流程

```
币安API → WebSocket → 连续性检测 → 质量标记 → 数据库存储 → API服务 → 前端展示
```

## 🚀 快速启动

### 1. 启动后端服务
```bash
# 启动市场数据服务
cargo run --bin market-data

# 启动API网关
cargo run --bin gateway

# 启动交易引擎
cargo run --bin trading-engine
```

### 2. 启动前端
```bash
cd frontend
npm install
npm run dev
```

## 📈 功能特性

### 市场数据服务
- ✅ 实时WebSocket数据采集
- ✅ K线连续性检测
- ✅ 数据质量标记系统
- ✅ 代理连接支持 (127.0.0.1:4780)

### 交易引擎
- ✅ 订单管理系统
- ✅ 持仓管理
- ✅ 风险控制框架
- 🔄 策略执行引擎

### 前端界面
- ✅ 专业交易界面
- ✅ 实时K线图表
- ✅ 市场数据展示
- ✅ 响应式设计

## � 配置说始明

### 环境变量
```bash
# 数据库存储开关
ENABLE_DATABASE_STORAGE=true

# 代理配置
PROXY_HOST=127.0.0.1
PROXY_PORT=4780
```

## 📝 开发日志

- **2024-12-17**: K线连续性检测Phase 1完成
- **2024-12-17**: DataQuality系统集成完成
- **2024-12-17**: 编译错误修复完成

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 📄 许可证

MIT License