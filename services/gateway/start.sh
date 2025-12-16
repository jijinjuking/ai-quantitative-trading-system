#!/bin/bash

# 企业级量化交易平台 - API网关启动脚本

set -e

echo "🚀 启动API网关服务..."

# 检查环境
echo "📋 检查环境依赖..."

# 检查Rust环境
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo未安装，请先安装Rust"
    exit 1
fi

# 检查Redis
if ! command -v redis-cli &> /dev/null; then
    echo "⚠️  Redis CLI未找到，请确保Redis已安装"
fi

# 检查Redis连接
if ! redis-cli ping &> /dev/null; then
    echo "🔄 启动Redis服务..."
    if command -v docker &> /dev/null; then
        docker run -d --name redis-gateway -p 6379:6379 redis:7-alpine || true
        sleep 2
    else
        echo "❌ Redis未运行且Docker不可用，请手动启动Redis"
        exit 1
    fi
fi

# 设置环境变量
if [ ! -f .env ]; then
    echo "📝 创建环境配置文件..."
    cp .env.example .env
    echo "⚠️  请编辑 .env 文件设置JWT_SECRET等配置"
fi

# 加载环境变量
export $(cat .env | grep -v '^#' | xargs)

# 编译项目
echo "🔨 编译项目..."
cargo build --release

# 启动服务
echo "🌟 启动网关服务..."
echo "📡 服务地址: http://${GATEWAY_HOST:-0.0.0.0}:${GATEWAY_PORT:-8080}"
echo "🏥 健康检查: http://${GATEWAY_HOST:-0.0.0.0}:${GATEWAY_PORT:-8080}/health"
echo "📊 指标监控: http://${GATEWAY_HOST:-0.0.0.0}:${GATEWAY_PORT:-8080}/metrics"
echo ""
echo "按 Ctrl+C 停止服务"
echo ""

# 运行服务
cargo run --release