#!/bin/bash

# 🛑 开发环境停止脚本
# 优雅关闭所有服务并保存状态

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

# 停止市场数据服务
stop_market_data_service() {
    log_info "停止市场数据服务..."
    
    if [ -f "data/market-data.pid" ]; then
        SERVICE_PID=$(cat data/market-data.pid)
        
        if kill -0 $SERVICE_PID 2>/dev/null; then
            log_info "发送优雅关机信号到进程 $SERVICE_PID"
            
            # 发送SIGTERM信号进行优雅关机
            kill -TERM $SERVICE_PID
            
            # 等待进程结束
            log_info "等待服务优雅关机..."
            timeout=30
            while [ $timeout -gt 0 ] && kill -0 $SERVICE_PID 2>/dev/null; do
                sleep 1
                timeout=$((timeout - 1))
            done
            
            if kill -0 $SERVICE_PID 2>/dev/null; then
                log_warning "优雅关机超时，强制终止进程"
                kill -KILL $SERVICE_PID
            else
                log_success "服务已优雅关闭"
            fi
        else
            log_warning "进程 $SERVICE_PID 不存在"
        fi
        
        rm -f data/market-data.pid
    else
        log_warning "未找到服务PID文件"
    fi
}

# 检查服务状态
check_final_state() {
    log_info "检查最终服务状态..."
    
    if [ -f "data/service_state.json" ]; then
        if command -v jq &> /dev/null; then
            shutdown_time=$(jq -r '.shutdown_time' data/service_state.json)
            if [ "$shutdown_time" != "null" ]; then
                log_success "服务状态已保存，关闭时间: $(date -d @$((shutdown_time/1000)))"
            else
                log_warning "服务状态文件存在但未记录关闭时间"
            fi
        fi
    else
        log_warning "未找到服务状态文件"
    fi
}

# 停止基础设施服务
stop_infrastructure() {
    log_info "停止基础设施服务..."
    
    # 停止Docker Compose服务
    if [ -f "docker-compose.dev.yml" ]; then
        docker-compose -f docker-compose.dev.yml down
        log_success "基础设施服务已停止"
    else
        log_warning "未找到docker-compose.dev.yml文件"
    fi
}

# 清理临时文件
cleanup_temp_files() {
    log_info "清理临时文件..."
    
    # 清理PID文件
    rm -f data/*.pid
    
    # 清理日志文件（可选）
    if [ "$1" = "--clean-logs" ]; then
        log_info "清理日志文件..."
        rm -f logs/*.log
    fi
    
    log_success "临时文件清理完成"
}

# 显示数据统计
show_final_stats() {
    log_info "显示最终统计信息..."
    
    if [ -f "data/service_state.json" ] && command -v jq &> /dev/null; then
        echo
        echo "📊 服务运行统计:"
        
        startup_time=$(jq -r '.startup_time' data/service_state.json)
        shutdown_time=$(jq -r '.shutdown_time' data/service_state.json)
        symbols_count=$(jq -r '.last_processed_timestamps | length' data/service_state.json)
        
        if [ "$startup_time" != "null" ]; then
            echo "  - 启动时间: $(date -d @$((startup_time/1000)))"
        fi
        
        if [ "$shutdown_time" != "null" ]; then
            echo "  - 关闭时间: $(date -d @$((shutdown_time/1000)))"
            
            if [ "$startup_time" != "null" ]; then
                uptime_seconds=$(((shutdown_time - startup_time) / 1000))
                uptime_hours=$((uptime_seconds / 3600))
                uptime_minutes=$(((uptime_seconds % 3600) / 60))
                echo "  - 运行时长: ${uptime_hours}小时${uptime_minutes}分钟"
            fi
        fi
        
        echo "  - 监控交易对: $symbols_count 个"
        
        # 显示最后处理的时间戳
        echo "  - 最后处理的交易对:"
        jq -r '.last_processed_timestamps | to_entries[] | "    \(.key): \(.value | todate)"' data/service_state.json 2>/dev/null || true
    fi
}

# 备份重要数据
backup_data() {
    if [ "$1" = "--backup" ]; then
        log_info "备份重要数据..."
        
        backup_dir="backups/$(date +%Y%m%d_%H%M%S)"
        mkdir -p "$backup_dir"
        
        # 备份服务状态
        if [ -f "data/service_state.json" ]; then
            cp data/service_state.json "$backup_dir/"
        fi
        
        # 备份配置文件
        if [ -f "config/development.toml" ]; then
            cp config/development.toml "$backup_dir/"
        fi
        
        # 备份最近的日志
        if [ -f "logs/market-data.log" ]; then
            tail -n 1000 logs/market-data.log > "$backup_dir/market-data.log"
        fi
        
        log_success "数据已备份到: $backup_dir"
    fi
}

# 主函数
main() {
    echo "🛑 停止量化交易平台开发环境"
    echo "=================================="
    
    # 解析参数
    CLEAN_LOGS=false
    BACKUP_DATA=false
    
    for arg in "$@"; do
        case $arg in
            --clean-logs)
                CLEAN_LOGS=true
                ;;
            --backup)
                BACKUP_DATA=true
                ;;
            --help)
                echo "用法: $0 [选项]"
                echo "选项:"
                echo "  --clean-logs    清理日志文件"
                echo "  --backup        备份重要数据"
                echo "  --help          显示帮助信息"
                exit 0
                ;;
        esac
    done
    
    # 备份数据（如果需要）
    if [ "$BACKUP_DATA" = true ]; then
        backup_data --backup
    fi
    
    # 停止服务
    stop_market_data_service
    check_final_state
    show_final_stats
    
    # 停止基础设施
    stop_infrastructure
    
    # 清理文件
    if [ "$CLEAN_LOGS" = true ]; then
        cleanup_temp_files --clean-logs
    else
        cleanup_temp_files
    fi
    
    echo
    log_success "🎉 开发环境已完全停止！"
    echo
    echo "📋 下次启动:"
    echo "  - 恢复运行: ./scripts/start-dev-environment.sh"
    echo "  - 全新开始: ./scripts/reset-dev-environment.sh"
    echo
    echo "📁 数据保留:"
    echo "  - 服务状态: data/service_state.json"
    echo "  - 数据库数据: data/clickhouse/, data/redis/"
    if [ "$CLEAN_LOGS" = false ]; then
        echo "  - 日志文件: logs/"
    fi
    if [ "$BACKUP_DATA" = true ]; then
        echo "  - 备份数据: backups/"
    fi
    echo
}

# 错误处理
trap 'log_error "停止过程中发生错误"; exit 1' ERR

# 执行主函数
main "$@"