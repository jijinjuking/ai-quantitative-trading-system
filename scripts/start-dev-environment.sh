#!/bin/bash

# 🚀 开发环境启动脚本
# 自动处理数据连续性和故障恢复

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    # 检查Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker未安装"
        exit 1
    fi
    
    # 检查Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose未安装"
        exit 1
    fi
    
    # 检查Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust未安装"
        exit 1
    fi
    
    log_success "所有依赖检查通过"
}

# 创建必要目录
create_directories() {
    log_info "创建数据目录..."
    
    mkdir -p data/redis
    mkdir -p data/clickhouse
    mkdir -p data/kafka
    mkdir -p logs
    
    log_success "目录创建完成"
}

# 启动基础设施
start_infrastructure() {
    log_info "启动基础设施服务..."
    
    # 启动Docker服务
    docker-compose -f docker-compose.dev.yml up -d
    
    # 等待服务启动
    log_info "等待服务启动..."
    sleep 10
    
    # 检查Redis
    if ! docker exec $(docker-compose -f docker-compose.dev.yml ps -q redis) redis-cli ping > /dev/null 2>&1; then
        log_error "Redis启动失败"
        exit 1
    fi
    log_success "Redis启动成功"
    
    # 检查ClickHouse
    max_attempts=30
    attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:8123/ping > /dev/null 2>&1; then
            log_success "ClickHouse启动成功"
            break
        fi
        attempt=$((attempt + 1))
        sleep 2
    done
    
    if [ $attempt -eq $max_attempts ]; then
        log_error "ClickHouse启动超时"
        exit 1
    fi
    
    # 检查Kafka
    if ! docker-compose -f docker-compose.dev.yml ps kafka | grep -q "Up"; then
        log_error "Kafka启动失败"
        exit 1
    fi
    log_success "Kafka启动成功"
    
    log_success "所有基础设施服务启动完成"
}

# 初始化数据库
initialize_database() {
    log_info "初始化ClickHouse数据库..."
    
    # 创建数据库
    echo "CREATE DATABASE IF NOT EXISTS market_data" | docker exec -i $(docker-compose -f docker-compose.dev.yml ps -q clickhouse) clickhouse-client
    
    log_success "数据库初始化完成"
}

# 检查服务状态
check_service_status() {
    log_info "检查之前的服务状态..."
    
    if [ -f "data/service_state.json" ]; then
        log_info "发现之前的服务状态文件"
        
        # 显示状态信息
        if command -v jq &> /dev/null; then
            startup_time=$(jq -r '.startup_time' data/service_state.json)
            shutdown_time=$(jq -r '.shutdown_time' data/service_state.json)
            symbols_count=$(jq -r '.last_processed_timestamps | length' data/service_state.json)
            
            log_info "上次启动时间: $(date -d @$((startup_time/1000)))"
            if [ "$shutdown_time" != "null" ]; then
                log_info "上次关闭时间: $(date -d @$((shutdown_time/1000)))"
            fi
            log_info "监控的交易对数量: $symbols_count"
        fi
        
        log_warning "服务将从上次状态恢复"
    else
        log_info "未发现之前的状态，将全新启动"
    fi
}

# 编译项目
build_project() {
    log_info "编译项目..."
    
    cd services/market-data
    cargo build --release
    cd ../..
    
    log_success "项目编译完成"
}

# 启动市场数据服务
start_market_data_service() {
    log_info "启动市场数据服务..."
    
    # 设置环境变量
    export RUST_LOG=info
    export MARKET_DATA_CONFIG_PATH="config/development.toml"
    
    # 启动服务（后台运行）
    cd services/market-data
    nohup cargo run --release > ../../logs/market-data.log 2>&1 &
    SERVICE_PID=$!
    cd ../..
    
    # 保存PID
    echo $SERVICE_PID > data/market-data.pid
    
    log_info "服务PID: $SERVICE_PID"
    log_info "日志文件: logs/market-data.log"
    
    # 等待服务启动
    log_info "等待服务启动..."
    max_attempts=30
    attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:8081/health > /dev/null 2>&1; then
            log_success "市场数据服务启动成功"
            break
        fi
        attempt=$((attempt + 1))
        sleep 2
    done
    
    if [ $attempt -eq $max_attempts ]; then
        log_error "服务启动超时"
        log_error "请检查日志: tail -f logs/market-data.log"
        exit 1
    fi
}

# 验证服务功能
verify_service() {
    log_info "验证服务功能..."
    
    # 检查健康状态
    health_response=$(curl -s http://localhost:8081/health)
    if echo "$health_response" | grep -q '"success":true'; then
        log_success "健康检查通过"
    else
        log_error "健康检查失败"
        exit 1
    fi
    
    # 检查数据连续性
    if curl -s http://localhost:8081/api/v1/admin/continuity-stats > /dev/null 2>&1; then
        log_success "数据连续性API可用"
    else
        log_warning "数据连续性API不可用"
    fi
    
    log_success "服务功能验证完成"
}

# 显示服务信息
show_service_info() {
    log_success "🎉 开发环境启动完成！"
    echo
    echo "📊 服务信息:"
    echo "  - 市场数据服务: http://localhost:8081"
    echo "  - 健康检查: http://localhost:8081/health"
    echo "  - 详细健康检查: http://localhost:8081/health/detailed"
    echo "  - 指标: http://localhost:8081/metrics"
    echo
    echo "🔧 管理API:"
    echo "  - 数据统计: curl http://localhost:8081/api/v1/admin/stats"
    echo "  - 连续性统计: curl http://localhost:8081/api/v1/admin/continuity-stats"
    echo "  - 数据修复: curl -X POST http://localhost:8081/api/v1/admin/repair-data"
    echo
    echo "📋 基础设施:"
    echo "  - Redis: localhost:6379"
    echo "  - ClickHouse: localhost:8123"
    echo "  - Kafka: localhost:9092"
    echo
    echo "📝 日志和监控:"
    echo "  - 服务日志: tail -f logs/market-data.log"
    echo "  - 实时监控: ./scripts/monitor-service.sh"
    echo "  - 数据检查: ./scripts/check-gaps.sh BTCUSDT 1"
    echo
    echo "🛑 停止服务:"
    echo "  - 优雅关机: ./scripts/stop-dev-environment.sh"
    echo "  - 强制停止: kill $(cat data/market-data.pid)"
    echo
}

# 主函数
main() {
    echo "🚀 启动量化交易平台开发环境"
    echo "=================================="
    
    check_dependencies
    create_directories
    start_infrastructure
    initialize_database
    check_service_status
    build_project
    start_market_data_service
    verify_service
    show_service_info
    
    log_success "开发环境启动完成！"
}

# 错误处理
trap 'log_error "启动过程中发生错误，正在清理..."; docker-compose -f docker-compose.dev.yml down; exit 1' ERR

# 执行主函数
main "$@"