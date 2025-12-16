# Docker开发环境设置指南

## 概述

本项目提供了完整的Docker开发环境，解决了Windows系统上Rust编译依赖问题（如CMake、Perl、OpenSSL等）。

## 前置要求

1. **Docker Desktop** - 从 [官网](https://www.docker.com/products/docker-desktop/) 下载安装
2. **Docker Compose** - 通常随Docker Desktop一起安装

## 快速开始

### 1. 构建开发环境

```powershell
# 使用PowerShell脚本（推荐）
.\scripts\docker-dev.ps1 build

# 或使用批处理文件
.\scripts\docker-dev.bat build
```

### 2. 启动所有服务

```powershell
# 启动基础设施服务（Redis、ClickHouse、Kafka等）
.\scripts\docker-dev.ps1 up
```

### 3. 进入开发容器

```powershell
# 进入Rust开发环境
.\scripts\docker-dev.ps1 shell
```

### 4. 在容器中编译项目

```bash
# 在容器内执行
cargo build --release

# 或从外部执行
.\scripts\docker-dev.ps1 compile
```

## 服务访问地址

启动后可以通过以下地址访问各种服务：

- **Kafka UI**: http://localhost:8080 - Kafka管理界面
- **Redis Commander**: http://localhost:8081 - Redis管理界面  
- **ClickHouse HTTP**: http://localhost:8123 - ClickHouse HTTP接口

## 常用命令

### 管理脚本命令

```powershell
# 查看所有可用命令
.\scripts\docker-dev.ps1 help

# 构建开发环境镜像
.\scripts\docker-dev.ps1 build

# 启动所有服务
.\scripts\docker-dev.ps1 up

# 停止所有服务
.\scripts\docker-dev.ps1 down

# 重启所有服务
.\scripts\docker-dev.ps1 restart

# 进入Rust开发容器
.\scripts\docker-dev.ps1 shell

# 查看服务日志
.\scripts\docker-dev.ps1 logs

# 查看服务状态
.\scripts\docker-dev.ps1 status

# 在容器中编译项目
.\scripts\docker-dev.ps1 compile

# 在容器中运行测试
.\scripts\docker-dev.ps1 test

# 清理所有数据和镜像
.\scripts\docker-dev.ps1 clean
```

### 容器内开发

进入容器后，你可以像在本地一样使用Rust工具：

```bash
# 检查代码
cargo check

# 编译项目
cargo build

# 运行测试
cargo test

# 运行特定服务
cargo run --bin market-data

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 项目结构

```
23/
├── services/
│   ├── gateway/           # API网关服务
│   └── market-data/       # 市场数据服务
├── shared/
│   ├── models/           # 共享数据模型
│   ├── utils/            # 共享工具库
│   └── protocols/        # 通信协议
├── scripts/
│   ├── docker-dev.ps1    # PowerShell管理脚本
│   ├── docker-dev.bat    # 批处理管理脚本
│   └── docker-dev.sh     # Bash管理脚本（Linux/Mac）
├── config/               # 配置文件
├── data/                 # 数据目录（自动创建）
├── Dockerfile.dev        # 开发环境镜像
├── docker-compose.build.yml  # 完整开发环境
└── docker-compose.dev.yml    # 基础设施服务
```

## 开发工作流

### 1. 日常开发

```powershell
# 1. 启动环境（首次需要构建）
.\scripts\docker-dev.ps1 build
.\scripts\docker-dev.ps1 up

# 2. 进入开发容器
.\scripts\docker-dev.ps1 shell

# 3. 在容器内开发
cargo check
cargo build
cargo test

# 4. 完成后停止服务
.\scripts\docker-dev.ps1 down
```

### 2. 调试和监控

```powershell
# 查看服务状态
.\scripts\docker-dev.ps1 status

# 查看特定服务日志
.\scripts\docker-dev.ps1 logs kafka

# 查看所有服务日志
.\scripts\docker-dev.ps1 logs
```

### 3. 数据管理

- **Redis数据**: 访问 http://localhost:8081 使用Redis Commander
- **Kafka消息**: 访问 http://localhost:8080 使用Kafka UI
- **ClickHouse数据**: 访问 http://localhost:8123 或使用客户端连接

## 故障排除

### 1. 端口冲突

如果遇到端口冲突，可以修改 `docker-compose.build.yml` 中的端口映射：

```yaml
ports:
  - "6380:6379"  # 将Redis端口改为6380
```

### 2. 磁盘空间不足

```powershell
# 清理未使用的Docker资源
docker system prune -f

# 完全清理项目环境
.\scripts\docker-dev.ps1 clean
```

### 3. 容器启动失败

```powershell
# 查看详细日志
.\scripts\docker-dev.ps1 logs

# 重新构建镜像
.\scripts\docker-dev.ps1 build
```

### 4. 编译错误

在容器环境中，所有编译依赖都已预装，如果仍有问题：

```bash
# 在容器内更新依赖
cargo update

# 清理并重新编译
cargo clean
cargo build
```

## 性能优化

### 1. 缓存优化

项目使用Docker卷来缓存Cargo注册表和编译产物：

- `rust_cache`: Cargo注册表缓存
- `rust_target`: 编译产物缓存

### 2. 资源限制

可以在 `docker-compose.build.yml` 中为服务设置资源限制：

```yaml
services:
  rust-dev:
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2'
```

## 生产部署

开发环境配置不适用于生产环境。生产部署请参考：

- `docker-compose.prod.yml`（待创建）
- Kubernetes配置（待创建）
- 云服务部署指南（待创建）

## 支持

如果遇到问题：

1. 查看本文档的故障排除部分
2. 检查Docker Desktop是否正常运行
3. 确保有足够的磁盘空间和内存
4. 查看服务日志获取详细错误信息