#!/bin/bash

# 📊 实时服务监控脚本

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 检查服务是否运行
check_service_running() {
    if curl -s http://localhost:8081/health > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# 获取服务状态
get_service_status() {
    if check_service_running; then
        echo -e "${GREEN}🟢 RUNNING${NC}"
    else
        echo -e "${RED}🔴 STOPPED${NC}"
    fi
}

# 格式化数字
format_number() {
    printf "%'d" "$1" 2>/dev/null || echo "$1"
}

# 格式化百分比
format_percentage() {
    printf "%.2f%%" "$1" 2>/dev/null || echo "$1%"
}

# 显示服务信息
show_service_info() {
    clear
    echo -e "${CYAN}📊 量化交易平台 - 实时监控${NC}"
    echo -e "${CYAN}================================${NC}"
    echo "🕐 更新时间: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "🔄 刷新间隔: 5秒 (按 Ctrl+C 退出)"
    echo

    # 服务状态
    echo -e "${BLUE}🏥 服务状态${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -n "状态: "
    get_service_status

    if ! check_service_running; then
        echo -e "${RED}❌ 服务未运行，请先启动服务${NC}"
        echo "启动命令: ./scripts/start-dev-environment.sh"
        return
    fi

    # 获取健康状态
    health_response=$(curl -s http://localhost:8081/health/detailed 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$health_response" ]; then
        status=$(echo "$health_response" | jq -r '.data.status' 2>/dev/null || echo "unknown")
        uptime=$(echo "$health_response" | jq -r '.data.uptime_seconds' 2>/dev/null || echo "0")
        version=$(echo "$health_response" | jq -r '.data.version' 2>/dev/null || echo "unknown")
        
        # 转换运行时间
        if [ "$uptime" != "0" ] && [ "$uptime" != "null" ]; then
            hours=$((uptime / 3600))
            minutes=$(((uptime % 3600) / 60))
            seconds=$((uptime % 60))
            uptime_str="${hours}h ${minutes}m ${seconds}s"
        else
            uptime_str="未知"
        fi
        
        echo "健康状态: $status"
        echo "运行时间: $uptime_str"
        echo "版本: $version"
    fi
    echo

    # 处理统计
    echo -e "${BLUE}📈 处理统计${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    stats_response=$(curl -s http://localhost:8081/api/v1/admin/stats 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$stats_response" ]; then
        total_events=$(echo "$stats_response" | jq -r '.data.total_events_processed' 2>/dev/null || echo "0")
        events_per_sec=$(echo "$stats_response" | jq -r '.data.events_per_second' 2>/dev/null || echo "0")
        error_rate=$(echo "$stats_response" | jq -r '.data.error_rate' 2>/dev/null || echo "0")
        success_rate=$(echo "$stats_response" | jq -r '.data.success_rate' 2>/dev/null || echo "0")
        
        echo "总处理事件: $(format_number $total_events)"
        echo "处理速率: $events_per_sec 事件/秒"
        echo "成功率: $(format_percentage $success_rate)"
        echo "错误率: $(format_percentage $error_rate)"
    else
        echo "❌ 无法获取处理统计"
    fi
    echo

    # 连续性统计
    echo -e "${BLUE}🔗 数据连续性${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    continuity_response=$(curl -s http://localhost:8081/api/v1/admin/continuity-stats 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$continuity_response" ]; then
        gaps_detected=$(echo "$continuity_response" | jq -r '.data.total_gaps_detected' 2>/dev/null || echo "0")
        gaps_filled=$(echo "$continuity_response" | jq -r '.data.total_gaps_filled' 2>/dev/null || echo "0")
        active_symbols=$(echo "$continuity_response" | jq -r '.data.active_symbols' 2>/dev/null || echo "0")
        
        echo "检测到的间隙: $(format_number $gaps_detected)"
        echo "已修复间隙: $(format_number $gaps_filled)"
        echo "活跃交易对: $(format_number $active_symbols)"
        
        # 计算修复率
        if [ "$gaps_detected" != "0" ] && [ "$gaps_detected" != "null" ]; then
            repair_rate=$(echo "scale=2; $gaps_filled * 100 / $gaps_detected" | bc -l 2>/dev/null || echo "0")
            echo "修复率: $(format_percentage $repair_rate)"
        fi
    else
        echo "❌ 无法获取连续性统计"
    fi
    echo

    # 基础设施状态
    echo -e "${BLUE}🏗️ 基础设施${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Redis状态
    if docker exec $(docker-compose -f docker-compose.dev.yml ps -q redis 2>/dev/null) redis-cli ping > /dev/null 2>&1; then
        echo -e "Redis: ${GREEN}🟢 运行中${NC}"
    else
        echo -e "Redis: ${RED}🔴 停止${NC}"
    fi
    
    # ClickHouse状态
    if curl -s http://localhost:8123/ping > /dev/null 2>&1; then
        echo -e "ClickHouse: ${GREEN}🟢 运行中${NC}"
    else
        echo -e "ClickHouse: ${RED}🔴 停止${NC}"
    fi
    
    # Kafka状态
    if docker-compose -f docker-compose.dev.yml ps kafka 2>/dev/null | grep -q "Up"; then
        echo -e "Kafka: ${GREEN}🟢 运行中${NC}"
    else
        echo -e "Kafka: ${RED}🔴 停止${NC}"
    fi
    echo

    # 最近的日志
    echo -e "${BLUE}📝 最近日志 (最后5行)${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if [ -f "logs/market-data.log" ]; then
        tail -n 5 logs/market-data.log | while IFS= read -r line; do
            # 根据日志级别着色
            if echo "$line" | grep -q "ERROR"; then
                echo -e "${RED}$line${NC}"
            elif echo "$line" | grep -q "WARN"; then
                echo -e "${YELLOW}$line${NC}"
            elif echo "$line" | grep -q "INFO"; then
                echo -e "${GREEN}$line${NC}"
            else
                echo "$line"
            fi
        done
    else
        echo "❌ 日志文件不存在"
    fi
    echo

    # 快捷操作提示
    echo -e "${CYAN}🔧 快捷操作${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "• 查看完整日志: tail -f logs/market-data.log"
    echo "• 检查数据间隙: ./scripts/check-gaps.sh BTCUSDT 1"
    echo "• 手动修复数据: curl -X POST http://localhost:8081/api/v1/admin/repair-data"
    echo "• 停止服务: ./scripts/stop-dev-environment.sh"
}

# 主循环
main() {
    # 检查依赖
    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}警告: 未安装jq，某些功能可能无法正常显示${NC}"
        echo "安装命令: sudo apt-get install jq  # Ubuntu/Debian"
        echo "          brew install jq         # macOS"
        echo
    fi

    if ! command -v bc &> /dev/null; then
        echo -e "${YELLOW}警告: 未安装bc，某些计算可能无法正常显示${NC}"
    fi

    # 显示初始信息
    echo -e "${CYAN}🚀 启动实时监控...${NC}"
    sleep 2

    # 主监控循环
    while true; do
        show_service_info
        sleep 5
    done
}

# 错误处理
trap 'echo -e "\n${YELLOW}监控已停止${NC}"; exit 0' INT TERM

# 执行主函数
main "$@"