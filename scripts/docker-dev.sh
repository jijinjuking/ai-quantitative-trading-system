#!/bin/bash

# Docker开发环境管理脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# 颜色输出
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

# 显示帮助信息
show_help() {
    echo "Docker开发环境管理脚本"
    echo ""
    echo "用法: $0 [命令]"
    echo ""
    echo "命令:"
    echo "  build     构建开发环境镜像"
    echo "  up        启动所有服务"
    echo "  down      停止所有服务"
    echo "  restart   重启所有服务"
    echo "  shell     进入Rust开发容器"
    echo "  logs      查看服务日志"
    echo "  status    查看服务状态"
    echo "  clean     清理所有数据和镜像"
    echo "  compile   在容器中编译项目"
    echo "  test      在容器中运行测试"
    echo "  help      显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 build && $0 up     # 构建并启动环境"
    echo "  $0 shell              # 进入开发容器"
    echo "  $0 compile            # 编译项目"
}

# 检查Docker是否安装
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker未安装，请先安装Docker"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose未安装，请先安装Docker Compose"
        exit 1
    fi
}

# 构建开发环境
build_env() {
    log_info "构建Rust开发环境镜像..."
    docker-compose -f docker-compose.build.yml build rust-dev
    log_success "开发环境镜像构建完成"
}

# 启动服务
start_services() {
    log_info "启动开发环境服务..."
    
    # 创建必要的目录
    mkdir -p data/{redis,clickhouse,kafka,zookeeper}
    mkdir -p config/clickhouse
    
    # 启动基础设施服务
    docker-compose -f docker-compose.build.yml up -d redis clickhouse zookeeper kafka
    
    log_info "等待服务启动..."
    sleep 10
    
    # 启动管理工具
    docker-compose -f docker-compose.build.yml up -d kafka-ui redis-commander
    
    log_success "所有服务已启动"
    log_info "服务访问地址:"
    log_info "  - Kafka UI: http://localhost:8080"
    log_info "  - Redis Commander: http://localhost:8081"
    log_info "  - ClickHouse HTTP: http://localhost:8123"
}

# 停止服务
stop_services() {
    log_info "停止所有服务..."
    docker-compose -f docker-compose.build.yml down
    log_success "所有服务已停止"
}

# 重启服务
restart_services() {
    log_info "重启所有服务..."
    stop_services
    start_services
}

# 进入开发容器
enter_shell() {
    log_info "进入Rust开发容器..."
    
    # 确保容器正在运行
    if ! docker-compose -f docker-compose.build.yml ps rust-dev | grep -q "Up"; then
        log_info "启动Rust开发容器..."
        docker-compose -f docker-compose.build.yml up -d rust-dev
        sleep 5
    fi
    
    docker-compose -f docker-compose.build.yml exec rust-dev bash
}

# 查看日志
show_logs() {
    local service=${1:-""}
    
    if [ -z "$service" ]; then
        log_info "显示所有服务日志..."
        docker-compose -f docker-compose.build.yml logs -f
    else
        log_info "显示 $service 服务日志..."
        docker-compose -f docker-compose.build.yml logs -f "$service"
    fi
}

# 查看服务状态
show_status() {
    log_info "服务状态:"
    docker-compose -f docker-compose.build.yml ps
    
    echo ""
    log_info "容器资源使用情况:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
}

# 清理环境
clean_env() {
    log_warning "这将删除所有容器、镜像和数据，确定要继续吗? (y/N)"
    read -r response
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        log_info "停止并删除所有容器..."
        docker-compose -f docker-compose.build.yml down -v --rmi all
        
        log_info "清理未使用的Docker资源..."
        docker system prune -f
        
        log_info "删除数据目录..."
        rm -rf data/
        
        log_success "环境清理完成"
    else
        log_info "取消清理操作"
    fi
}

# 编译项目
compile_project() {
    log_info "在容器中编译项目..."
    
    # 确保容器正在运行
    if ! docker-compose -f docker-compose.build.yml ps rust-dev | grep -q "Up"; then
        log_info "启动Rust开发容器..."
        docker-compose -f docker-compose.build.yml up -d rust-dev
        sleep 5
    fi
    
    # 编译项目
    docker-compose -f docker-compose.build.yml exec rust-dev cargo build --release
    
    if [ $? -eq 0 ]; then
        log_success "项目编译成功"
    else
        log_error "项目编译失败"
        exit 1
    fi
}

# 运行测试
run_tests() {
    log_info "在容器中运行测试..."
    
    # 确保容器正在运行
    if ! docker-compose -f docker-compose.build.yml ps rust-dev | grep -q "Up"; then
        log_info "启动Rust开发容器..."
        docker-compose -f docker-compose.build.yml up -d rust-dev
        sleep 5
    fi
    
    # 运行测试
    docker-compose -f docker-compose.build.yml exec rust-dev cargo test
    
    if [ $? -eq 0 ]; then
        log_success "所有测试通过"
    else
        log_error "测试失败"
        exit 1
    fi
}

# 主函数
main() {
    check_docker
    
    case "${1:-help}" in
        build)
            build_env
            ;;
        up)
            start_services
            ;;
        down)
            stop_services
            ;;
        restart)
            restart_services
            ;;
        shell)
            enter_shell
            ;;
        logs)
            show_logs "$2"
            ;;
        status)
            show_status
            ;;
        clean)
            clean_env
            ;;
        compile)
            compile_project
            ;;
        test)
            run_tests
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "未知命令: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"