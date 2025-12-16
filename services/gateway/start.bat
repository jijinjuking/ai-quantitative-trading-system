@echo off
REM 企业级量化交易平台 - API网关启动脚本

echo 🚀 启动API网关服务...

REM 检查环境
echo 📋 检查环境依赖...

REM 检查Rust环境
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Cargo未安装，请先安装Rust
    pause
    exit /b 1
)

REM 检查Redis连接
redis-cli ping >nul 2>nul
if %errorlevel% neq 0 (
    echo 🔄 启动Redis服务...
    where docker >nul 2>nul
    if %errorlevel% equ 0 (
        docker run -d --name redis-gateway -p 6379:6379 redis:7-alpine 2>nul
        timeout /t 2 >nul
    ) else (
        echo ❌ Redis未运行且Docker不可用，请手动启动Redis
        pause
        exit /b 1
    )
)

REM 设置环境变量
if not exist .env (
    echo 📝 创建环境配置文件...
    copy .env.example .env
    echo ⚠️  请编辑 .env 文件设置JWT_SECRET等配置
)

REM 编译项目
echo 🔨 编译项目...
cargo build --release

REM 启动服务
echo 🌟 启动网关服务...
echo 📡 服务地址: http://localhost:8080
echo 🏥 健康检查: http://localhost:8080/health
echo 📊 指标监控: http://localhost:8080/metrics
echo.
echo 按 Ctrl+C 停止服务
echo.

REM 运行服务
cargo run --release

pause