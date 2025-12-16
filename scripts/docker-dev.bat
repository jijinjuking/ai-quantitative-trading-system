@echo off
setlocal enabledelayedexpansion

REM Docker开发环境管理脚本 (Windows版本)

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."

cd /d "%PROJECT_DIR%"

REM 显示帮助信息
if "%1"=="help" goto :show_help
if "%1"=="" goto :show_help

REM 检查Docker是否安装
docker --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Docker未安装，请先安装Docker Desktop
    exit /b 1
)

docker-compose --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Docker Compose未安装，请先安装Docker Compose
    exit /b 1
)

REM 根据参数执行相应操作
if "%1"=="build" goto :build_env
if "%1"=="up" goto :start_services
if "%1"=="down" goto :stop_services
if "%1"=="restart" goto :restart_services
if "%1"=="shell" goto :enter_shell
if "%1"=="logs" goto :show_logs
if "%1"=="status" goto :show_status
if "%1"=="clean" goto :clean_env
if "%1"=="compile" goto :compile_project
if "%1"=="test" goto :run_tests

echo [ERROR] 未知命令: %1
goto :show_help

:show_help
echo Docker开发环境管理脚本
echo.
echo 用法: %0 [命令]
echo.
echo 命令:
echo   build     构建开发环境镜像
echo   up        启动所有服务
echo   down      停止所有服务
echo   restart   重启所有服务
echo   shell     进入Rust开发容器
echo   logs      查看服务日志
echo   status    查看服务状态
echo   clean     清理所有数据和镜像
echo   compile   在容器中编译项目
echo   test      在容器中运行测试
echo   help      显示此帮助信息
echo.
echo 示例:
echo   %0 build ^&^& %0 up     # 构建并启动环境
echo   %0 shell              # 进入开发容器
echo   %0 compile            # 编译项目
exit /b 0

:build_env
echo [INFO] 构建Rust开发环境镜像...
docker-compose -f docker-compose.build.yml build rust-dev
if errorlevel 1 (
    echo [ERROR] 镜像构建失败
    exit /b 1
)
echo [SUCCESS] 开发环境镜像构建完成
exit /b 0

:start_services
echo [INFO] 启动开发环境服务...

REM 创建必要的目录
if not exist "data" mkdir data
if not exist "data\redis" mkdir data\redis
if not exist "data\clickhouse" mkdir data\clickhouse
if not exist "data\kafka" mkdir data\kafka
if not exist "data\zookeeper" mkdir data\zookeeper
if not exist "config" mkdir config
if not exist "config\clickhouse" mkdir config\clickhouse

REM 启动基础设施服务
docker-compose -f docker-compose.build.yml up -d redis clickhouse zookeeper kafka
if errorlevel 1 (
    echo [ERROR] 服务启动失败
    exit /b 1
)

echo [INFO] 等待服务启动...
timeout /t 10 /nobreak >nul

REM 启动管理工具
docker-compose -f docker-compose.build.yml up -d kafka-ui redis-commander

echo [SUCCESS] 所有服务已启动
echo [INFO] 服务访问地址:
echo   - Kafka UI: http://localhost:8080
echo   - Redis Commander: http://localhost:8081
echo   - ClickHouse HTTP: http://localhost:8123
exit /b 0

:stop_services
echo [INFO] 停止所有服务...
docker-compose -f docker-compose.build.yml down
echo [SUCCESS] 所有服务已停止
exit /b 0

:restart_services
echo [INFO] 重启所有服务...
call :stop_services
call :start_services
exit /b 0

:enter_shell
echo [INFO] 进入Rust开发容器...

REM 检查容器是否运行
docker-compose -f docker-compose.build.yml ps rust-dev | findstr "Up" >nul
if errorlevel 1 (
    echo [INFO] 启动Rust开发容器...
    docker-compose -f docker-compose.build.yml up -d rust-dev
    timeout /t 5 /nobreak >nul
)

docker-compose -f docker-compose.build.yml exec rust-dev bash
exit /b 0

:show_logs
echo [INFO] 显示服务日志...
if "%2"=="" (
    docker-compose -f docker-compose.build.yml logs -f
) else (
    docker-compose -f docker-compose.build.yml logs -f %2
)
exit /b 0

:show_status
echo [INFO] 服务状态:
docker-compose -f docker-compose.build.yml ps
echo.
echo [INFO] 容器资源使用情况:
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
exit /b 0

:clean_env
echo [WARNING] 这将删除所有容器、镜像和数据，确定要继续吗? (y/N)
set /p response=
if /i "!response!"=="y" (
    echo [INFO] 停止并删除所有容器...
    docker-compose -f docker-compose.build.yml down -v --rmi all
    
    echo [INFO] 清理未使用的Docker资源...
    docker system prune -f
    
    echo [INFO] 删除数据目录...
    if exist "data" rmdir /s /q data
    
    echo [SUCCESS] 环境清理完成
) else (
    echo [INFO] 取消清理操作
)
exit /b 0

:compile_project
echo [INFO] 在容器中编译项目...

REM 检查容器是否运行
docker-compose -f docker-compose.build.yml ps rust-dev | findstr "Up" >nul
if errorlevel 1 (
    echo [INFO] 启动Rust开发容器...
    docker-compose -f docker-compose.build.yml up -d rust-dev
    timeout /t 5 /nobreak >nul
)

REM 编译项目
docker-compose -f docker-compose.build.yml exec rust-dev cargo build --release
if errorlevel 1 (
    echo [ERROR] 项目编译失败
    exit /b 1
)
echo [SUCCESS] 项目编译成功
exit /b 0

:run_tests
echo [INFO] 在容器中运行测试...

REM 检查容器是否运行
docker-compose -f docker-compose.build.yml ps rust-dev | findstr "Up" >nul
if errorlevel 1 (
    echo [INFO] 启动Rust开发容器...
    docker-compose -f docker-compose.build.yml up -d rust-dev
    timeout /t 5 /nobreak >nul
)

REM 运行测试
docker-compose -f docker-compose.build.yml exec rust-dev cargo test
if errorlevel 1 (
    echo [ERROR] 测试失败
    exit /b 1
)
echo [SUCCESS] 所有测试通过
exit /b 0