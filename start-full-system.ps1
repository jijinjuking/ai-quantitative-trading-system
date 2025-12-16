# 启动完整的Rust微服务交易系统
# PowerShell脚本

Write-Host "🚀 启动企业级量化交易平台..." -ForegroundColor Green

# 检查Docker是否运行
Write-Host "📋 检查Docker状态..." -ForegroundColor Yellow
try {
    docker version | Out-Null
    Write-Host "✅ Docker运行正常" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker未运行，请先启动Docker Desktop" -ForegroundColor Red
    exit 1
}

# 启动基础设施服务
Write-Host "🏗️ 启动基础设施服务 (ClickHouse, Redis, Kafka)..." -ForegroundColor Yellow
docker-compose -f docker-compose.dev.yml up -d

# 等待基础设施启动
Write-Host "⏳ 等待基础设施服务启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# 检查基础设施状态
Write-Host "📊 检查基础设施状态..." -ForegroundColor Yellow
$services = @("clickhouse", "redis", "kafka")
foreach ($service in $services) {
    $status = docker-compose -f docker-compose.dev.yml ps $service --format "table {{.State}}"
    if ($status -match "running") {
        Write-Host "✅ $service 运行正常" -ForegroundColor Green
    } else {
        Write-Host "⚠️ $service 状态异常" -ForegroundColor Yellow
    }
}

# 启动Rust微服务
Write-Host "🦀 启动Rust微服务..." -ForegroundColor Yellow

# 启动网关服务
Write-Host "🌐 启动API网关服务 (端口8080)..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd services/gateway; Write-Host '🌐 API网关服务启动中...' -ForegroundColor Cyan; cargo run"

# 等待网关启动
Start-Sleep -Seconds 3

# 启动市场数据服务
Write-Host "📈 启动市场数据服务 (端口8083)..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd services/market-data; Write-Host '📈 市场数据服务启动中...' -ForegroundColor Cyan; cargo run"

# 等待市场数据服务启动
Start-Sleep -Seconds 3

# 启动交易引擎服务
Write-Host "⚡ 启动交易引擎服务 (端口8082)..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd services/trading-engine; Write-Host '⚡ 交易引擎服务启动中...' -ForegroundColor Cyan; cargo run"

# 等待交易引擎启动
Start-Sleep -Seconds 3

# 启动策略引擎服务
Write-Host "🧠 启动策略引擎服务 (端口8084)..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd services/strategy-engine; Write-Host '🧠 策略引擎服务启动中...' -ForegroundColor Cyan; cargo run"

# 等待所有服务启动
Write-Host "⏳ 等待所有微服务启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# 启动前端
Write-Host "🎨 启动前端应用 (端口3000)..." -ForegroundColor Magenta
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd frontend; Write-Host '🎨 前端应用启动中...' -ForegroundColor Magenta; npm run dev"

# 等待前端启动
Start-Sleep -Seconds 5

Write-Host ""
Write-Host "🎉 系统启动完成！" -ForegroundColor Green
Write-Host ""
Write-Host "📋 服务访问地址:" -ForegroundColor Yellow
Write-Host "  🌐 前端界面:     http://localhost:3000" -ForegroundColor White
Write-Host "  🔗 API网关:      http://localhost:8080" -ForegroundColor White
Write-Host "  📊 Kafka UI:     http://localhost:8080" -ForegroundColor White
Write-Host "  🗄️ Redis管理:    http://localhost:8081" -ForegroundColor White
Write-Host "  📈 ClickHouse:   http://localhost:8123" -ForegroundColor White
Write-Host ""
Write-Host "🔧 服务端口分配:" -ForegroundColor Yellow
Write-Host "  🌐 网关服务:     8080" -ForegroundColor White
Write-Host "  👤 用户管理:     8081" -ForegroundColor White
Write-Host "  ⚡ 交易引擎:     8082" -ForegroundColor White
Write-Host "  📈 市场数据:     8083" -ForegroundColor White
Write-Host "  🧠 策略引擎:     8084" -ForegroundColor White
Write-Host "  🛡️ 风险管理:     8085" -ForegroundColor White
Write-Host "  📢 通知服务:     8086" -ForegroundColor White
Write-Host "  📊 分析服务:     8087" -ForegroundColor White
Write-Host ""
Write-Host "💡 提示:" -ForegroundColor Yellow
Write-Host "  - 所有服务将在独立的PowerShell窗口中运行" -ForegroundColor White
Write-Host "  - 可以通过关闭对应窗口来停止服务" -ForegroundColor White
Write-Host "  - 前端会自动打开浏览器" -ForegroundColor White
Write-Host "  - 系统状态可在前端界面右上角查看" -ForegroundColor White
Write-Host ""
Write-Host "🚀 享受专业的量化交易体验！" -ForegroundColor Green